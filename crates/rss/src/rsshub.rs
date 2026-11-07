//! RSSHub routes: stored as rsshub://, resolved against the configured instance at fetch time.
use reqwest::Url;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

pub fn validate_instance(instance: &str) -> WyrmResult<Option<String>> {
    let instance = instance.trim();

    if instance.is_empty() {
        return Ok(None);
    }

    let url = Url::parse(instance).map_err(|_| invalid_instance())?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid_instance());
    }

    Ok(Some(url.as_str().trim_end_matches('/').to_string()))
}

fn invalid_instance() -> WyrmError {
    WyrmError::InvalidInput(
        "RSSHub instance must be an HTTP(S) base URL without credentials, query or fragment".into(),
    )
}

pub fn is_rsshub_url(url: &str) -> bool {
    normalize_url(url).is_ok()
}

/// Converts an HTTP(S) URL on `instance` to `rsshub://route`; None if it isn't on the instance.
pub fn from_instance_url(url: &str, instance: &str) -> Option<String> {
    let route = url.strip_prefix(instance)?.strip_prefix('/')?;
    (!route.is_empty() && !route.starts_with(['?', '#'])).then(|| format!("{SCHEME_PREFIX}{route}"))
}

const SCHEME_PREFIX: &str = "rsshub://";

/// Canonical `rsshub://route` form; errors on non-RSSHub URLs.
/// Only the scheme and leading slashes change; the route is case-sensitive.
pub fn normalize_url(url: &str) -> WyrmResult<String> {
    // Do not parse the custom URL as an authority: its first component is a
    // case-sensitive route namespace, not a hostname.
    match url.trim().split_once("://") {
        Some((scheme, route)) if scheme.eq_ignore_ascii_case("rsshub") => {
            Ok(format!("{SCHEME_PREFIX}{}", route.trim_start_matches('/')))
        }
        _ => Err(WyrmError::InvalidInput("Not an rsshub:// URL".into())),
    }
}

/// Converts `rsshub://route` to an HTTP(S) URL on `instance`.
/// Errors on non-RSSHub URLs, if no instance is set, or if the route is empty or escapes it.
pub fn resolve_url(url: &str, instance: Option<&str>) -> WyrmResult<String> {
    let normalized = normalize_url(url)?;
    let route = &normalized[SCHEME_PREFIX.len()..];

    // None and a blank instance both mean "not configured".
    let Some(instance) = validate_instance(instance.unwrap_or_default())? else {
        return Err(WyrmError::InvalidInput(
            "Set an RSSHub instance in Settings before adding rsshub:// feeds".into(),
        ));
    };

    if route.is_empty() || route.starts_with(['?', '#']) {
        return Err(WyrmError::InvalidInput(
            "RSSHub route cannot be empty".into(),
        ));
    }

    let base = Url::parse(&format!("{instance}/")).map_err(|_| invalid_instance())?;

    let resolved = Url::parse(&format!("{instance}/{route}"))
        .map_err(|_| WyrmError::InvalidInput("Invalid RSSHub route".into()))?;

    if resolved.origin() != base.origin() || !resolved.path().starts_with(base.path()) {
        return Err(WyrmError::InvalidInput(
            "RSSHub route must stay within the configured instance".into(),
        ));
    }
    Ok(resolved.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_routes_preserving_case_queries_and_escapes() {
        assert_eq!(
            resolve_url(
                "rsshub://GitHub/issue/DIYgod/RSSHub?label=help+wanted&filter=%252F#recent",
                Some("https://example.com/rsshub/")
            )
            .unwrap(),
            "https://example.com/rsshub/GitHub/issue/DIYgod/RSSHub?label=help+wanted&filter=%252F#recent"
        );
        assert_eq!(
            resolve_url("RSSHUB:///github/issue/a/b", Some("http://localhost:1200")).unwrap(),
            "http://localhost:1200/github/issue/a/b"
        );
    }

    #[test]
    fn normalizes_scheme_and_slashes_but_keeps_route_case() {
        assert_eq!(
            normalize_url("  RSSHUB:///GitHub/issue/DIYgod/RSSHub?label=x ").unwrap(),
            "rsshub://GitHub/issue/DIYgod/RSSHub?label=x"
        );
        assert!(normalize_url("https://example.com/feed.xml").is_err());
        assert!(normalize_url("not a url").is_err());
    }

    #[test]
    fn rejects_non_rsshub_urls() {
        assert!(resolve_url("https://example.com/feed.xml", Some("https://example.com")).is_err());
    }

    #[test]
    fn rejects_missing_instance_empty_route_and_path_escape() {
        assert!(resolve_url("rsshub://github/issue/a/b", None).is_err());
        for route in [
            "",
            "?key=x",
            "#fragment",
            "../feed",
            "%2e%2e/feed",
            "..\\feed",
        ] {
            assert!(
                resolve_url(
                    &format!("rsshub://{route}"),
                    Some("https://example.com/rsshub")
                )
                .is_err()
            );
        }
    }

    #[test]
    fn validates_instance_and_allows_clearing_it() {
        assert_eq!(validate_instance("  ").unwrap(), None);
        assert_eq!(
            validate_instance(" https://example.com/rsshub/ ").unwrap(),
            Some("https://example.com/rsshub".into())
        );
        for instance in [
            "garbage",
            "file:///tmp/rss",
            "https://user:pass@example.com",
            "https://example.com?key=x",
            "https://example.com#x",
        ] {
            assert!(validate_instance(instance).is_err());
        }
    }
}
