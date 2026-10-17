use feed_rs::model::Entry;

#[derive(Debug, PartialEq)]
enum FilterScope {
    All,
    Url,
    Title,
    Content,
}

#[derive(Debug, PartialEq)]
enum FilterAction {
    /// `+` prefix: keep only entries matching at least one include filter.
    Include,
    /// No prefix: drop entries matching this filter. Wins over includes.
    Exclude,
}

/// Splits a stored filter into its scope, action and pattern: a leading `+`
/// marks an include filter. An optional `url:` / `title:` / `content:` prefix
/// limits the match to that field, no prefix matches every field. Only known
/// prefixes count as scopes, e.g. "https://example.com" as a filter excludes
/// posts where that URL appears anywhere (url, title or content).
fn parse_filter(raw: &str) -> (FilterScope, FilterAction, &str) {
    let (action, rest) = match raw.strip_prefix('+') {
        Some(rest) => (FilterAction::Include, rest),
        None => (FilterAction::Exclude, raw),
    };

    match rest.split_once(':') {
        Some(("url", p)) => (FilterScope::Url, action, p),
        Some(("title", p)) => (FilterScope::Title, action, p),
        Some(("content", p)) => (FilterScope::Content, action, p),
        _ => (FilterScope::All, action, rest),
    }
}

/// A feed's include and exclude filters, parsed once per poll and matched per entry.
/// Matching is case-insensitive: patterns are lowercased here, haystacks in
/// [`CompiledFilters::excludes`].
pub struct CompiledFilters(Vec<(FilterScope, FilterAction, String)>);

impl CompiledFilters {
    pub fn new(filters: &[Option<String>]) -> Self {
        Self(
            filters
                .iter()
                .filter_map(|f| f.as_deref())
                .map(|raw| {
                    let (scope, action, pattern) = parse_filter(raw);
                    (scope, action, pattern.to_lowercase())
                })
                .collect(),
        )
    }

    /// True when the entry should be skipped: it matches an exclude filter, or
    /// the feed has include filters and it matches none of them.
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
        let matches = |(scope, _, pattern): &(FilterScope, FilterAction, String)| match scope {
            FilterScope::Url => url.contains(pattern.as_str()),
            FilterScope::Title => title.contains(pattern.as_str()),
            FilterScope::Content => {
                summary.contains(pattern.as_str()) || body.contains(pattern.as_str())
            }
            FilterScope::All => [url, title, summary, body]
                .into_iter()
                .any(|h| h.contains(pattern.as_str())),
        };

        let has_includes = self
            .0
            .iter()
            .any(|(_, action, _)| *action == FilterAction::Include);

        let included = self
            .0
            .iter()
            .any(|f| f.1 == FilterAction::Include && matches(f));

        let excluded = self
            .0
            .iter()
            .any(|f| f.1 == FilterAction::Exclude && matches(f));

        excluded || (has_includes && !included)
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
        assert_eq!(
            parse_filter("url:/shorts"),
            (FilterScope::Url, FilterAction::Exclude, "/shorts")
        );
        assert_eq!(
            parse_filter("title:sponsored"),
            (FilterScope::Title, FilterAction::Exclude, "sponsored")
        );
        assert_eq!(
            parse_filter("content:crypto"),
            (FilterScope::Content, FilterAction::Exclude, "crypto")
        );
    }

    #[test]
    fn parse_filter_bare_pattern_matches_all() {
        assert_eq!(
            parse_filter("sponsored"),
            (FilterScope::All, FilterAction::Exclude, "sponsored")
        );
    }

    #[test]
    fn parse_filter_unknown_prefix_keeps_colon() {
        assert_eq!(
            parse_filter("https://example.com"),
            (
                FilterScope::All,
                FilterAction::Exclude,
                "https://example.com"
            )
        );
        assert_eq!(
            parse_filter("titel:typo"),
            (FilterScope::All, FilterAction::Exclude, "titel:typo")
        );
    }

    #[test]
    fn parse_filter_plus_prefix_marks_include() {
        assert_eq!(
            parse_filter("+url:/shorts"),
            (FilterScope::Url, FilterAction::Include, "/shorts")
        );
        assert_eq!(
            parse_filter("+sponsored"),
            (FilterScope::All, FilterAction::Include, "sponsored")
        );
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

    #[test]
    fn include_skips_entries_that_do_not_match() {
        let f = compiled(&["+url:/shorts"]);
        assert!(!f.excludes_fields("https://youtube.com/shorts/abc", "", "", ""));
        assert!(f.excludes_fields("https://youtube.com/watch?v=1", "", "", ""));
    }

    #[test]
    fn matching_any_include_is_enough() {
        let f = compiled(&["+title:rust", "+title:zig"]);
        assert!(!f.excludes_fields("", "rust 2.0 released", "", ""));
        assert!(!f.excludes_fields("", "zig 1.0 released", "", ""));
        assert!(f.excludes_fields("", "go 2.0 released", "", ""));
    }

    #[test]
    fn exclude_wins_over_include() {
        let f = compiled(&["+title:rust", "title:jobs"]);
        assert!(!f.excludes_fields("", "rust 2.0 released", "", ""));
        assert!(f.excludes_fields("", "rust jobs this week", "", ""));
    }
}
