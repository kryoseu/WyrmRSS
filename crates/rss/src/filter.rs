use feed_rs::model::Entry;

#[derive(Debug, PartialEq)]
enum FilterScope {
    All,
    Url,
    Title,
    Content,
}

/// Splits a stored filter into its scope and pattern: an optional `url:` /
/// `title:` / `content:` prefix limits the match to that field, no prefix
/// matches every field. Only known prefixes count as scopes — a bare
/// "https://example.com" keeps its colon and stays a plain pattern.
fn parse_filter(raw: &str) -> (FilterScope, &str) {
    match raw.split_once(':') {
        Some(("url", p)) => (FilterScope::Url, p),
        Some(("title", p)) => (FilterScope::Title, p),
        Some(("content", p)) => (FilterScope::Content, p),
        _ => (FilterScope::All, raw),
    }
}

/// A feed's exclusion filters, parsed once per poll and matched per entry.
/// Matching is case-insensitive: patterns are lowercased here, haystacks in
/// [`CompiledFilters::excludes`].
pub struct CompiledFilters(Vec<(FilterScope, String)>);

impl CompiledFilters {
    pub fn new(filters: &[Option<String>]) -> Self {
        Self(
            filters
                .iter()
                .filter_map(|f| f.as_deref())
                .map(|raw| {
                    let (scope, pattern) = parse_filter(raw);
                    (scope, pattern.to_lowercase())
                })
                .collect(),
        )
    }

    /// True when any filter matches the entry, i.e. the entry should be skipped.
    pub fn excludes(&self, entry: &Entry) -> bool {
        if self.0.is_empty() {
            return false;
        }
        let url = entry
            .links
            .first()
            .map(|l| l.href.to_lowercase())
            .unwrap_or_default();
        let title = entry
            .title
            .as_ref()
            .map(|t| t.content.to_lowercase())
            .unwrap_or_default();
        let summary = entry
            .summary
            .as_ref()
            .map(|s| s.content.to_lowercase())
            .unwrap_or_default();
        let body = entry
            .content
            .as_ref()
            .and_then(|c| c.body.as_deref())
            .map(str::to_lowercase)
            .unwrap_or_default();
        self.excludes_fields(&url, &title, &summary, &body)
    }

    // Content scope covers both summary and body: feeds routinely fill only
    // one of the two (from_entry stores summary as the description for the
    // same reason).
    fn excludes_fields(&self, url: &str, title: &str, summary: &str, body: &str) -> bool {
        self.0.iter().any(|(scope, pattern)| match scope {
            FilterScope::Url => url.contains(pattern),
            FilterScope::Title => title.contains(pattern),
            FilterScope::Content => summary.contains(pattern) || body.contains(pattern),
            FilterScope::All => [url, title, summary, body]
                .into_iter()
                .any(|h| h.contains(pattern)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compiled(raw: &[&str]) -> CompiledFilters {
        let owned: Vec<Option<String>> = raw.iter().map(|f| Some(f.to_string())).collect();
        CompiledFilters::new(&owned)
    }

    #[test]
    fn parse_filter_scoped_prefixes() {
        assert_eq!(parse_filter("url:/shorts"), (FilterScope::Url, "/shorts"));
        assert_eq!(
            parse_filter("title:sponsored"),
            (FilterScope::Title, "sponsored")
        );
        assert_eq!(
            parse_filter("content:crypto"),
            (FilterScope::Content, "crypto")
        );
    }

    #[test]
    fn parse_filter_bare_pattern_matches_all() {
        assert_eq!(parse_filter("sponsored"), (FilterScope::All, "sponsored"));
    }

    #[test]
    fn parse_filter_unknown_prefix_keeps_colon() {
        assert_eq!(
            parse_filter("https://example.com"),
            (FilterScope::All, "https://example.com")
        );
        assert_eq!(parse_filter("titel:typo"), (FilterScope::All, "titel:typo"));
    }

    #[test]
    fn url_scope_ignores_other_fields() {
        let f = compiled(&["url:/shorts"]);
        assert!(f.excludes_fields("https://youtube.com/shorts/abc", "a title", "", ""));
        assert!(!f.excludes_fields(
            "https://youtube.com/watch?v=1",
            "watch /shorts later",
            "/shorts",
            ""
        ));
    }

    #[test]
    fn content_scope_matches_summary_or_body() {
        let f = compiled(&["content:crypto"]);
        assert!(f.excludes_fields("", "", "all about crypto", ""));
        assert!(f.excludes_fields("", "", "", "<p>crypto</p>"));
        assert!(!f.excludes_fields("https://crypto.example", "crypto news", "", ""));
    }

    #[test]
    fn bare_pattern_matches_any_field() {
        // excludes_fields expects pre-lowercased haystacks, as produced by excludes.
        let f = compiled(&["sponsored"]);
        assert!(f.excludes_fields("", "sponsored: new gadget", "", ""));
        assert!(f.excludes_fields("", "", "", "this post is sponsored"));
        assert!(!f.excludes_fields("https://example.com", "a title", "text", "body"));
    }

    #[test]
    fn matching_is_case_insensitive() {
        let f = compiled(&["title:SPONSORED"]);
        assert!(f.excludes_fields("", "sponsored content", "", ""));
    }

    #[test]
    fn no_filters_excludes_nothing() {
        let f = CompiledFilters::new(&[]);
        assert!(!f.excludes_fields("url", "title", "summary", "body"));
    }
}
