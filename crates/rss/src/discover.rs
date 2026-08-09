//! Feed URL discovery: turns a user-supplied page URL into a feed URL.
//!
//! A submitted URL that already serves RSS/Atom passes through unchanged.
//! For HTML pages we look for the standard autodiscovery tag
//! (`<link rel="alternate" type="application/rss+xml" href="…">`), which
//! covers most blogs as well as YouTube channel pages in all their URL
//! shapes (`/@handle`, `/channel/…`, `/c/…`, `/user/…`). Two YouTube
//! extras: `/channel/<id>` URLs are rewritten without a network round-trip,
//! and pages served without the autodiscovery tag (consent walls, degraded
//! bot HTML) fall back to scraping the embedded `"channelId"`.

use crate::http::HttpClient;
use scraper::{Html, Selector};
use tracing::warn;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

/// What a submitted URL turned out to serve — fetched once by [`fetch_url`],
/// then shared by feed-URL resolution and icon resolution.
pub enum FetchedUrl {
    /// The URL served a feed. Boxed: the parsed feed is ~1 KB, and the enum
    /// is sized to its largest variant; the other variants shouldn't carry
    /// that weight.
    Feed(Box<feed_rs::model::Feed>),
    /// The URL served something else (typically an HTML page); the body is
    /// kept for autodiscovery scanning.
    Html(String),
    /// Nothing fetched: host unreachable (down is not invalid; YouTube and
    /// Reddit feeds have dead windows, and the poller retries forever), or
    /// the URL resolves without a network round-trip (YouTube `/channel/`
    /// shortcut).
    None,
}

impl FetchedUrl {
    /// The parsed feed, when the URL served one — feeds icon resolution.
    pub fn parsed_feed(&self) -> Option<&feed_rs::model::Feed> {
        match self {
            FetchedUrl::Feed(feed) => Some(feed.as_ref()),
            _ => None,
        }
    }
}

/// Fetches a submitted URL once and classifies what it serves.
pub async fn fetch_url(http: &HttpClient, url: &str) -> FetchedUrl {
    // `/channel/<id>` URLs resolve directly; skip the network round-trip.
    if youtube_channel_shortcut(url).is_some() {
        return FetchedUrl::None;
    }

    let bytes = match http.fetch(url).await {
        Ok((bytes, _)) => bytes,
        Err(e) => {
            warn!("feed discovery: could not fetch {url}: {e}");
            return FetchedUrl::None;
        }
    };

    match feed_rs::parser::parse(&bytes[..]) {
        Ok(parsed) => FetchedUrl::Feed(Box::new(parsed)),
        Err(_) => FetchedUrl::Html(String::from_utf8_lossy(&bytes).into_owned()),
    }
}

/// Resolves `url` to the feed URL to store, judging by what [`fetch_url`]
/// found there. Fails with a client-facing error only on positive evidence:
/// the page fetched fine and advertises no feed. An unfetched URL passes
/// through unchanged.
pub fn resolve_feed_url(url: &str, fetched: &FetchedUrl) -> WyrmResult<String> {
    if let Some(feed) = youtube_channel_shortcut(url) {
        return Ok(feed);
    }

    let body = match fetched {
        FetchedUrl::Feed(_) | FetchedUrl::None => return Ok(url.to_string()),
        FetchedUrl::Html(body) => body,
    };

    if let Some(href) = find_alternate_link(body) {
        return absolutize(url, &href);
    }
    if let Some(id) = find_channel_id(body) {
        return Ok(youtube_feed_url(id));
    }

    Err(WyrmError::FeedDiscovery(format!("no feed found at {url}")))
}

fn youtube_feed_url(channel_id: &str) -> String {
    format!("https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}")
}

/// `youtube.com/channel/<id>` already contains the channel id, so the feed
/// URL can be built directly. A malformed id falls through to the fetch
/// path instead of silently creating a feed that would 404 at poll time.
fn youtube_channel_shortcut(url: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(url).ok()?;
    let host = parsed.host_str()?;
    if !matches!(host, "www.youtube.com" | "youtube.com" | "m.youtube.com") {
        return None;
    }
    let mut segments = parsed.path_segments()?;
    if segments.next()? != "channel" {
        return None;
    }
    let id = segments.next()?;
    is_channel_id(id).then(|| youtube_feed_url(id))
}

/// Channel ids are exactly 24 URL-safe-base64 characters starting with "UC".
fn is_channel_id(id: &str) -> bool {
    id.starts_with("UC")
        && id.len() == 24
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Finds the first RSS/Atom autodiscovery `<link>` in an HTML document and
/// returns its `href`.
fn find_alternate_link(html: &str) -> Option<String> {
    let selector = Selector::parse(
        r#"link[rel~="alternate" i][type="application/rss+xml" i],
           link[rel~="alternate" i][type="application/atom+xml" i]"#,
    )
    .expect("static selector is valid");

    Html::parse_document(html)
        .select(&selector)
        .find_map(|link| link.value().attr("href").map(String::from))
}

/// YouTube pages embed `"channelId":"UC…"` in their initial-data JSON even
/// when the autodiscovery tag is missing.
fn find_channel_id(html: &str) -> Option<&str> {
    const KEY: &str = "\"channelId\":\"";
    let start = html.find(KEY)? + KEY.len();
    let rest = &html[start..];
    let id = &rest[..rest.find('"')?];
    is_channel_id(id).then_some(id)
}

/// Resolves a discovered `href` (possibly relative) against the page URL.
fn absolutize(base: &str, href: &str) -> WyrmResult<String> {
    reqwest::Url::parse(base)
        .and_then(|b| b.join(href))
        .map(String::from)
        .map_err(|_| WyrmError::FeedDiscovery(format!("invalid feed link discovered at {base}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_judges_by_fetched_classification() {
        // A URL that already serves a feed (or couldn't be fetched) passes
        // through unchanged.
        let rss = feed_rs::parser::parse(
            &br#"<rss version="2.0"><channel><title>t</title></channel></rss>"#[..],
        )
        .expect("minimal rss should parse");
        assert_eq!(
            resolve_feed_url("https://example.com/feed", &FetchedUrl::Feed(Box::new(rss))).unwrap(),
            "https://example.com/feed"
        );
        assert_eq!(
            resolve_feed_url("https://example.com/feed", &FetchedUrl::None).unwrap(),
            "https://example.com/feed"
        );

        // An HTML page resolves through its autodiscovery link; one that
        // advertises no feed is the only hard failure.
        let html = r#"<link rel="alternate" type="application/rss+xml" href="/feed.xml">"#;
        assert_eq!(
            resolve_feed_url("https://example.com/blog", &FetchedUrl::Html(html.into())).unwrap(),
            "https://example.com/feed.xml"
        );
        assert!(
            resolve_feed_url("https://example.com", &FetchedUrl::Html("<p>hi</p>".into())).is_err()
        );

        // The /channel/ shortcut wins regardless of what was (not) fetched.
        assert_eq!(
            resolve_feed_url(
                "https://www.youtube.com/channel/UCVBlOjOg74sx8Gk8Zjmjyrg",
                &FetchedUrl::None
            )
            .unwrap(),
            "https://www.youtube.com/feeds/videos.xml?channel_id=UCVBlOjOg74sx8Gk8Zjmjyrg"
        );
    }

    #[test]
    fn channel_url_rewrites_without_fetching() {
        for url in [
            "https://www.youtube.com/channel/UCVBlOjOg74sx8Gk8Zjmjyrg",
            "https://youtube.com/channel/UCVBlOjOg74sx8Gk8Zjmjyrg/videos",
            "https://m.youtube.com/channel/UCVBlOjOg74sx8Gk8Zjmjyrg",
        ] {
            assert_eq!(
                youtube_channel_shortcut(url).as_deref(),
                Some(
                    "https://www.youtube.com/feeds/videos.xml?channel_id=UCVBlOjOg74sx8Gk8Zjmjyrg"
                )
            );
        }
    }

    #[test]
    fn non_channel_urls_are_not_shortcut() {
        assert_eq!(
            youtube_channel_shortcut("https://www.youtube.com/@handle"),
            None
        );
        assert_eq!(
            youtube_channel_shortcut("https://example.com/channel/UC123"),
            None
        );
        assert_eq!(youtube_channel_shortcut("not a url"), None);
        // A malformed id must fall through to the fetch path, not build a
        // feed URL that would 404 at poll time.
        assert_eq!(
            youtube_channel_shortcut("https://www.youtube.com/channel/garbage"),
            None
        );
    }

    #[test]
    fn finds_alternate_link_regardless_of_attribute_order_or_case() {
        let youtube_style = r#"<html><head><link rel="alternate" type="application/rss+xml" title="RSS" href="https://www.youtube.com/feeds/videos.xml?channel_id=UCabc"></head></html>"#;
        assert_eq!(
            find_alternate_link(youtube_style).as_deref(),
            Some("https://www.youtube.com/feeds/videos.xml?channel_id=UCabc")
        );

        let reordered = r#"<LINK HREF='/feed.xml' TYPE='application/ATOM+xml' REL='Alternate'/>"#;
        assert_eq!(find_alternate_link(reordered).as_deref(), Some("/feed.xml"));
    }

    #[test]
    fn ignores_stylesheets_and_decodes_entities() {
        let html = r#"
            <link rel="stylesheet" href="/style.css">
            <link rel="alternate" type="application/rss+xml" href="/feed?a=1&amp;b=2">
        "#;
        assert_eq!(find_alternate_link(html).as_deref(), Some("/feed?a=1&b=2"));
        assert_eq!(find_alternate_link("<p>no links here</p>"), None);
    }

    #[test]
    fn channel_id_fallback_validates_shape() {
        let html = r#"{"channelId":"UCVBlOjOg74sx8Gk8Zjmjyrg","title":"x"}"#;
        assert_eq!(find_channel_id(html), Some("UCVBlOjOg74sx8Gk8Zjmjyrg"));

        // Wrong prefix, wrong length, or bad characters are rejected.
        assert_eq!(
            find_channel_id(r#"{"channelId":"XXVBlOjOg74sx8Gk8Zjmjyrg"}"#),
            None
        );
        assert_eq!(find_channel_id(r#"{"channelId":"UCshort"}"#), None);
        assert_eq!(
            find_channel_id(r#"{"channelId":"UCVBlOjOg74sx8Gk8Zjm.yrg"}"#),
            None
        );
    }

    #[test]
    fn absolutize_resolves_relative_hrefs() {
        assert_eq!(
            absolutize("https://example.com/blog/post", "/feed.xml").unwrap(),
            "https://example.com/feed.xml"
        );
        assert_eq!(
            absolutize("https://example.com/", "https://other.com/rss").unwrap(),
            "https://other.com/rss"
        );
        assert!(absolutize("nonsense", "/feed.xml").is_err());
    }
}
