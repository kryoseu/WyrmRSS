//! Resolve RSSHub routes before discovery so stored feeds use ordinary HTTP URLs.
use reqwest::Url;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

pub fn validate_instance(value: &str) -> WyrmResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(String::new());
    }
    let url = Url::parse(value).map_err(|_| invalid_instance())?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid_instance());
    }
    Ok(url.as_str().trim_end_matches('/').to_string())
}

fn invalid_instance() -> WyrmError {
    WyrmError::InvalidInput(
        "RSSHub instance must be an HTTP(S) base URL without credentials, query or fragment".into(),
    )
}

pub fn resolve_url(value: &str, instance: &str) -> WyrmResult<String> {
    let value = value.trim();
    // Do not parse the custom URL as an authority: its first component is a
    // case-sensitive route namespace, not a hostname.
    let Some((scheme, route)) = value.split_once("://") else {
        return Ok(value.to_string());
    };
    if !scheme.eq_ignore_ascii_case("rsshub") {
        return Ok(value.to_string());
    }
    let instance = validate_instance(instance)?;
    if instance.is_empty() {
        return Err(WyrmError::InvalidInput(
            "Set an RSSHub instance in Settings before adding rsshub:// feeds".into(),
        ));
    }
    let route = route.trim_start_matches('/');
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
                "https://example.com/rsshub/"
            )
            .unwrap(),
            "https://example.com/rsshub/GitHub/issue/DIYgod/RSSHub?label=help+wanted&filter=%252F#recent"
        );
        assert_eq!(
            resolve_url("RSSHUB:///github/issue/a/b", "http://localhost:1200").unwrap(),
            "http://localhost:1200/github/issue/a/b"
        );
    }

    #[test]
    fn ordinary_feeds_do_not_require_an_instance() {
        let url = "https://example.com/feed.xml?a=%2F#latest";
        assert_eq!(resolve_url(url, "").unwrap(), url);
    }

    #[test]
    fn rejects_missing_instance_empty_route_and_path_escape() {
        assert!(resolve_url("rsshub://github/issue/a/b", "").is_err());
        for route in [
            "",
            "?key=x",
            "#fragment",
            "../feed",
            "%2e%2e/feed",
            "..\\feed",
        ] {
            assert!(
                resolve_url(&format!("rsshub://{route}"), "https://example.com/rsshub").is_err()
            );
        }
    }

    #[test]
    fn validates_instance_and_allows_clearing_it() {
        assert_eq!(validate_instance("  ").unwrap(), "");
        assert_eq!(
            validate_instance(" https://example.com/rsshub/ ").unwrap(),
            "https://example.com/rsshub"
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
