//! Feed URL handling: the form to store and the URL to fetch, per runtime settings.
use crate::rsshub;
use database::utils::settings::RuntimeSettings;
use reqwest::Url;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[derive(Debug, Clone, Default)]
pub struct FeedUrlResolver {
    rsshub_instance: Option<String>,
}

impl From<&RuntimeSettings> for FeedUrlResolver {
    fn from(s: &RuntimeSettings) -> Self {
        FeedUrlResolver {
            rsshub_instance: s.rsshub_instance.clone(),
        }
    }
}

impl FeedUrlResolver {
    /// The URL to request: rsshub:// resolves against the configured instance;
    /// plain URLs are fetched as is.
    pub fn resolve(&self, url: &str) -> WyrmResult<String> {
        if rsshub::is_rsshub_url(url) {
            rsshub::resolve_url(url, self.rsshub_instance.as_deref())
        } else {
            Ok(url.trim().to_string())
        }
    }

    /// The URL to store: canonical rsshub:// form (also for URLs on the instance),
    /// or a normalized plain URL.
    pub fn normalize(&self, url: &str) -> WyrmResult<String> {
        if rsshub::is_rsshub_url(url) {
            return rsshub::normalize_url(url);
        }

        let parsed = Url::parse(url.trim())
            .map_err(|_| WyrmError::InvalidInput("Invalid feed URL".into()))?;

        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(WyrmError::InvalidInput("Feed URL must be HTTP(S)".into()));
        }
        let url: String = parsed.into();

        if let Some(instance) = &self.rsshub_instance
            && let Some(rsshub_url) = rsshub::from_instance_url(&url, instance)
        {
            Ok(rsshub_url)
        } else {
            Ok(url)
        }
    }

    /// Whether the feed can be fetched with the current settings.
    pub fn is_pollable(&self, url: &str) -> bool {
        if rsshub::is_rsshub_url(url) {
            self.rsshub_instance.is_some()
        } else {
            true
        }
    }

    /// URLs that must serve a feed directly, so HTML discovery is skipped.
    pub fn skips_discovery(&self, url: &str) -> bool {
        rsshub::is_rsshub_url(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_instance(instance: Option<&str>) -> FeedUrlResolver {
        FeedUrlResolver {
            rsshub_instance: instance.map(String::from),
        }
    }

    #[test]
    fn resolves_rsshub_and_passes_plain_urls_through() {
        let enabled = with_instance(Some("https://example.com/rsshub"));
        assert_eq!(
            enabled.resolve("rsshub://github/issue/a/b").unwrap(),
            "https://example.com/rsshub/github/issue/a/b"
        );
        let plain = "https://example.com/feed.xml?a=%2F#latest";
        assert_eq!(with_instance(None).resolve(plain).unwrap(), plain);
        assert!(
            with_instance(None)
                .resolve("rsshub://github/issue/a/b")
                .is_err()
        );
    }

    #[test]
    fn normalizes_rsshub_and_plain_urls() {
        let resolver = with_instance(None);
        assert_eq!(
            resolver.normalize(" RSSHUB:///GitHub/x ").unwrap(),
            "rsshub://GitHub/x"
        );
        assert_eq!(
            resolver
                .normalize(" HTTPS://Example.COM:443/Feed.xml?Q=1 ")
                .unwrap(),
            "https://example.com/Feed.xml?Q=1"
        );
        for invalid in [
            "example.com/feed",
            "ftp://example.com/feed",
            "file:///etc/passwd",
        ] {
            assert!(resolver.normalize(invalid).is_err());
        }
    }

    #[test]
    fn stores_urls_on_the_instance_as_rsshub_routes() {
        let enabled = with_instance(Some("https://example.com/rsshub"));
        assert_eq!(
            enabled
                .normalize("HTTPS://Example.com/rsshub/github/issue/a/b")
                .unwrap(),
            "rsshub://github/issue/a/b"
        );
        for plain in [
            "https://example.com/rsshub/",
            "https://other.com/rsshub/github/issue/a/b",
            "https://example.com/rsshubx/feed",
        ] {
            assert!(enabled.normalize(plain).unwrap().starts_with("https://"));
        }
        assert_eq!(
            with_instance(None)
                .normalize("https://example.com/rsshub/github/issue/a/b")
                .unwrap(),
            "https://example.com/rsshub/github/issue/a/b"
        );
    }

    #[test]
    fn rsshub_feeds_are_pollable_only_with_an_instance() {
        let enabled = with_instance(Some("https://example.com/rsshub"));
        let disabled = with_instance(None);
        assert!(enabled.is_pollable("rsshub://github/issue/a/b"));
        assert!(!disabled.is_pollable("rsshub://github/issue/a/b"));
        assert!(disabled.is_pollable("https://example.com/feed.xml"));
    }
}
