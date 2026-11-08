//! Picks a thumbnail image for a feed entry, shown on post cards.

use feed_rs::model::Entry;
use scraper::{Html, Selector};

/// First image found, in order: media:thumbnail, image media:content or
/// enclosure, Atom image enclosure link, first `<img>` in summary, then content.
pub fn extract_thumbnail_url(entry: &Entry) -> Option<String> {
    let media_thumbnail = entry
        .media
        .iter()
        .flat_map(|m| &m.thumbnails)
        .map(|t| t.image.uri.clone())
        .next();

    media_thumbnail
        .or_else(|| {
            entry
                .media
                .iter()
                .flat_map(|m| &m.content)
                .find(|c| c.content_type.as_ref().is_some_and(|t| t.ty() == "image"))
                .and_then(|c| c.url.as_ref().map(|u| u.to_string()))
        })
        .or_else(|| {
            entry
                .links
                .iter()
                .find(|l| {
                    l.rel.as_deref() == Some("enclosure")
                        && l.media_type
                            .as_deref()
                            .is_some_and(|t| t.starts_with("image/"))
                })
                .map(|l| l.href.clone())
        })
        .or_else(|| {
            entry
                .summary
                .as_ref()
                .and_then(|s| first_img_src(&s.content))
        })
        .or_else(|| {
            entry
                .content
                .as_ref()
                .and_then(|c| c.body.as_deref())
                .and_then(first_img_src)
        })
}

fn first_img_src(html: &str) -> Option<String> {
    let selector = Selector::parse("img[src]").expect("valid selector");
    Html::parse_fragment(html)
        .select(&selector)
        .next()
        .and_then(|img| img.value().attr("src"))
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use feed_rs::parser;

    fn first_entry(xml: &str) -> Entry {
        parser::parse(xml.as_bytes())
            .expect("fixture should parse")
            .entries
            .into_iter()
            .next()
            .expect("fixture should contain an entry")
    }

    fn atom_entry(entry_body: &str) -> Entry {
        first_entry(&format!(
            concat!(
                r#"<?xml version="1.0" encoding="utf-8"?>"#,
                r#"<feed xmlns="http://www.w3.org/2005/Atom" xmlns:media="http://search.yahoo.com/mrss/">"#,
                "<title>Test Feed</title><id>urn:feed</id><updated>2026-01-01T00:00:00Z</updated>",
                "<entry><id>urn:1</id><title>Post</title>{}</entry>",
                "</feed>",
            ),
            entry_body,
        ))
    }

    fn rss_item(item_body: &str) -> Entry {
        first_entry(&format!(
            concat!(
                r#"<?xml version="1.0" encoding="utf-8"?>"#,
                r#"<rss version="2.0"><channel><title>Test Feed</title>"#,
                "<item><title>Post</title>{}</item>",
                "</channel></rss>",
            ),
            item_body,
        ))
    }

    #[test]
    fn prefers_media_thumbnail_over_other_sources() {
        let entry = atom_entry(concat!(
            "<media:group>",
            r#"<media:content url="https://example.com/full.jpg" type="image/jpeg"/>"#,
            r#"<media:thumbnail url="https://example.com/thumb.jpg"/>"#,
            "</media:group>",
            r#"<summary type="html"><![CDATA[<img src="https://example.com/inline.jpg">]]></summary>"#,
        ));

        assert_eq!(
            extract_thumbnail_url(&entry).as_deref(),
            Some("https://example.com/thumb.jpg")
        );
    }

    #[test]
    fn uses_rss_image_enclosure() {
        let entry = rss_item(
            r#"<enclosure url="https://example.com/cover.jpg" type="image/jpeg" length="1234"/>"#,
        );

        assert_eq!(
            extract_thumbnail_url(&entry).as_deref(),
            Some("https://example.com/cover.jpg")
        );
    }

    #[test]
    fn skips_podcast_audio_enclosure() {
        let entry = rss_item(
            r#"<enclosure url="https://example.com/episode.mp3" type="audio/mpeg" length="1234"/>"#,
        );

        assert_eq!(extract_thumbnail_url(&entry), None);
    }

    #[test]
    fn uses_atom_image_enclosure_link() {
        let entry = atom_entry(
            r#"<link rel="enclosure" type="image/webp" href="https://example.com/hero.webp"/>"#,
        );

        assert_eq!(
            extract_thumbnail_url(&entry).as_deref(),
            Some("https://example.com/hero.webp")
        );
    }

    #[test]
    fn uses_first_img_in_summary() {
        let entry = atom_entry(concat!(
            r#"<summary type="html"><![CDATA[<p>Intro</p><img src="https://example.com/first.jpg">"#,
            r#"<img src="https://example.com/second.jpg">]]></summary>"#,
        ));

        assert_eq!(
            extract_thumbnail_url(&entry).as_deref(),
            Some("https://example.com/first.jpg")
        );
    }

    #[test]
    fn falls_back_to_img_in_content_when_summary_has_none() {
        let entry = atom_entry(concat!(
            "<summary>Plain text summary.</summary>",
            r#"<content type="html"><![CDATA[<p>Body</p><img src="https://example.com/body.jpg">]]></content>"#,
        ));

        assert_eq!(
            extract_thumbnail_url(&entry).as_deref(),
            Some("https://example.com/body.jpg")
        );
    }

    #[test]
    fn returns_none_without_any_image() {
        let entry = atom_entry(concat!(
            "<summary>Just text.</summary>",
            r#"<content type="html"><![CDATA[<p>No pictures here.</p>]]></content>"#,
        ));

        assert_eq!(extract_thumbnail_url(&entry), None);
    }
}
