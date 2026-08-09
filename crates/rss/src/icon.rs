//! Feed icon resolution: finds a small image to represent a feed in the UI.
//!
//! The resolution order:
//!  1. The feed's own `<icon>`/`<logo>`;
//!  2. The `<link rel="icon">` tags of the site page the feed links to;
//!  3. `/favicon.ico` at the site origin.
//!
//! Every candidate is fetched and validated before it is stored,
//! so a 404 page served for a missing favicon never ends up in the database.

use crate::http::HttpClient;
use chrono::{Duration, Utc};
use database::{
    DatabasePool,
    models::{
        feed::Feed,
        feed_icon::{FeedIcon, FeedIconInsertForm},
    },
};
use scraper::{Html, Selector};
use tracing::warn;

/// Favicons run a few KB.
/// Anything larger is not an icon worth storing and serving per feed.
const MAX_ICON_BYTES: usize = 256 * 1024;

/// How long a "checked, nothing found" result suppresses re-resolution.
const RETRY_MISSING_AFTER: Duration = Duration::days(7);

/// Brings `feed`'s stored icon up to date: resolves and stores one if the
/// feed was never checked, or a previous check found nothing more than
/// [`RETRY_MISSING_AFTER`] ago. Best-effort by design — failures are logged,
/// never propagated, because a missing favicon must not fail feed creation
/// or a poll. Callers without a parsed feed at hand simply skip the call:
/// the next poll parses one and lands here anyway.
pub async fn ensure_feed_icon(
    pool: &DatabasePool,
    http: &HttpClient,
    feed: &Feed,
    parsed: &feed_rs::model::Feed,
) {
    let existing = match FeedIcon::get(pool, feed.id).await {
        Ok(existing) => existing,
        Err(_) => return,
    };

    // Found icons are kept until the feed's URL changes (the row is deleted
    // then); recent misses wait out the retry window.
    if existing.is_some_and(|icon| {
        icon.data.is_some() || Utc::now() - icon.checked_at < RETRY_MISSING_AFTER
    }) {
        return;
    }

    let (data, content_type) = resolve_icon(http, &feed.url, parsed).await.unzip();
    let form = FeedIconInsertForm {
        feed_id: feed.id,
        data,
        content_type,
    };
    if let Err(e) = FeedIcon::create(pool, form).await {
        warn!("feed icon: could not store result for {}: {e}", feed.url);
    }
}

/// Runs the source cascade and returns the first candidate that fetches and
/// validates as an image, as `(bytes, mime)`.
async fn resolve_icon(
    http: &HttpClient,
    feed_url: &str,
    parsed: &feed_rs::model::Feed,
) -> Option<(Vec<u8>, String)> {
    // The feed's own icon/logo elements.
    for image in [parsed.icon.as_ref(), parsed.logo.as_ref()]
        .into_iter()
        .flatten()
    {
        if let Some(icon) = fetch_icon(http, feed_url, &image.uri).await {
            return Some(icon);
        }
    }

    // The site page the feed links to.
    let page_url = site_url(feed_url, parsed);
    if let Some(page_url) = &page_url
        && let Ok((bytes, _)) = http.fetch(page_url).await
    {
        let body = String::from_utf8_lossy(&bytes);
        for href in page_icon_candidates(&body) {
            if let Some(icon) = fetch_icon(http, page_url, &href).await {
                return Some(icon);
            }
        }
    }

    // Most sites serve /favicon.ico even without advertising it.
    fetch_icon(
        http,
        page_url.as_deref().unwrap_or(feed_url),
        "/favicon.ico",
    )
    .await
}

/// The site page to scan for icons: the feed's first non-self link (the
/// channel/home page for YouTube and most blogs), or `None` when the feed
/// links nowhere and only the origin favicon fallback remains.
fn site_url(feed_url: &str, parsed: &feed_rs::model::Feed) -> Option<String> {
    parsed
        .links
        .iter()
        .find(|link| link.rel.as_deref() != Some("self"))
        .and_then(|link| join_url(feed_url, &link.href))
}

/// Icon URL candidates advertised by an HTML page, in document order.
/// `og:image` is deliberately not considered: it is a share banner (or, on
/// YouTube, the channel avatar), not the site's icon.
fn page_icon_candidates(html: &str) -> Vec<String> {
    // rel is a space-separated token list, so ~= also matches "shortcut icon".
    let selector = Selector::parse(r#"link[rel~="icon" i], link[rel~="apple-touch-icon" i]"#)
        .expect("static selector is valid");
    Html::parse_document(html)
        .select(&selector)
        .filter_map(|link| link.value().attr("href").map(String::from))
        .collect()
}

async fn fetch_icon(http: &HttpClient, base: &str, href: &str) -> Option<(Vec<u8>, String)> {
    let url = join_url(base, href)?;
    let (bytes, content_type) = http.fetch(&url).await.ok()?;
    if bytes.is_empty() || bytes.len() > MAX_ICON_BYTES {
        return None;
    }
    let mime = sniff_image(&bytes)
        .map(String::from)
        .or(content_type.filter(|ct| ct.starts_with("image/")))?;
    Some((bytes.to_vec(), mime))
}

/// Identifies an image format from its leading bytes.
/// https://en.wikipedia.org/wiki/List_of_file_signatures
fn sniff_image(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if bytes.starts_with(b"\xff\xd8\xff") {
        return Some("image/jpeg");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if bytes.starts_with(b"\x00\x00\x01\x00") {
        return Some("image/x-icon");
    }
    if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP".as_slice()) {
        return Some("image/webp");
    }
    // SVG has no magic number; accept only documents that open as SVG/XML,
    // not arbitrary text that merely mentions an <svg> tag somewhere.
    let head = std::str::from_utf8(&bytes[..bytes.len().min(1024)]).ok()?;
    let head = head.trim_start_matches('\u{feff}').trim_start();
    if head.starts_with("<svg") || (head.starts_with("<?xml") && head.contains("<svg")) {
        return Some("image/svg+xml");
    }
    None
}

/// Resolves a possibly-relative `href` against `base`.
fn join_url(base: &str, href: &str) -> Option<String> {
    reqwest::Url::parse(base)
        .and_then(|b| b.join(href))
        .map(String::from)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_common_formats_and_rejects_garbage() {
        assert_eq!(sniff_image(b"\x89PNG\r\n\x1a\n...."), Some("image/png"));
        assert_eq!(sniff_image(b"\xff\xd8\xff\xe0...."), Some("image/jpeg"));
        assert_eq!(sniff_image(b"GIF89a...."), Some("image/gif"));
        assert_eq!(sniff_image(b"\x00\x00\x01\x00...."), Some("image/x-icon"));
        assert_eq!(
            sniff_image(b"RIFF\x00\x00\x00\x00WEBPVP8 "),
            Some("image/webp")
        );
        assert_eq!(
            sniff_image(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>"),
            Some("image/svg+xml")
        );
        assert_eq!(
            sniff_image(b"<?xml version=\"1.0\"?><svg/>"),
            Some("image/svg+xml")
        );

        assert_eq!(sniff_image(b""), None);
        assert_eq!(sniff_image(b"<html><body>404</body></html>"), None);
        // Text mentioning <svg> mid-document is not an SVG.
        assert_eq!(sniff_image(b"<html>an inline <svg/> here</html>"), None);
    }

    #[test]
    fn page_candidates_cover_icon_rel_variants_in_order() {
        let html = r#"
            <link rel="stylesheet" href="/style.css">
            <link rel="shortcut icon" href="/favicon.ico">
            <LINK REL="Icon" HREF="/favicon-32.png" SIZES="32x32">
            <link rel="apple-touch-icon" href="/touch.png">
            <meta property="og:image" content="/banner.png">
        "#;
        // og:image is a share banner (or channel avatar), not an icon.
        assert_eq!(
            page_icon_candidates(html),
            vec!["/favicon.ico", "/favicon-32.png", "/touch.png"]
        );
    }

    #[test]
    fn join_url_resolves_relative_and_absolute() {
        assert_eq!(
            join_url("https://example.com/blog/feed.xml", "/favicon.ico").as_deref(),
            Some("https://example.com/favicon.ico")
        );
        assert_eq!(
            join_url("https://example.com/", "https://cdn.example.com/icon.png").as_deref(),
            Some("https://cdn.example.com/icon.png")
        );
        assert_eq!(join_url("nonsense", "/favicon.ico"), None);
    }
}
