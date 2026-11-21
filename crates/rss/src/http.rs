use bytes::{Bytes, BytesMut};
use database::{
    models::{
        feed::Feed,
        host_credential::{HostCredential, HostCredentialKind},
    },
    utils::settings::RuntimeSettings,
};
use http::{Extensions, HeaderName, HeaderValue};
use reqwest::{Request, Response, StatusCode, Url, header};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next, Result};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use std::{collections::HashMap, time::Duration};
use tracing::{debug, info, warn};
use wyrm_utils::{
    error::{HttpClientError, WyrmError},
    result::WyrmResult,
};

/// Hard cap on fetched bodies: full-content feeds run a few MB; anything
/// larger is not a feed (or a page advertising one). Applied to bytes after
/// transparent gzip decompression, so oversized *and* highly-compressed
/// responses both bail instead of exhausting memory.
const MAX_RESPONSE_BYTES: usize = 10 * 1024 * 1024;

struct LoggingMiddleware;

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        info!("Req: {}", req.url());
        next.run(req, extensions).await
    }
}

#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Response timeout
    pub timeout: u64,
    //// Conn timeout
    pub connect_timeout: u64,
    /// Max retries
    pub retries: u32,
    /// User Agent header
    pub user_agent: Option<String>,
}

impl From<&RuntimeSettings> for HttpConfig {
    fn from(s: &RuntimeSettings) -> Self {
        HttpConfig {
            timeout: s.http_timeout as u64,
            connect_timeout: s.http_connect_timeout as u64,
            retries: s.http_retries as u32,
            user_agent: s.http_user_agent.clone(),
        }
    }
}

#[derive(Default)]
pub struct HttpValidators {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

impl From<&Feed> for HttpValidators {
    fn from(feed: &Feed) -> Self {
        Self {
            etag: feed.etag.clone(),
            last_modified: feed.last_modified.clone(),
        }
    }
}

/// Feed polling can skip parsing and storage when the publisher returns 304.
pub enum FeedResponse {
    Modified {
        body: Bytes,
        validators: HttpValidators,
    },
    NotModified(HttpValidators),
}

#[derive(Clone)]
pub struct HttpClient {
    inner: ClientWithMiddleware,
    host_credentials: HashMap<String, HostCredential>,
}

impl HttpClient {
    pub fn new(config: &HttpConfig, credentials: Vec<HostCredential>) -> WyrmResult<HttpClient> {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(config.retries);

        let user_agent = config.user_agent.as_deref().unwrap_or(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ));

        let timeout = Duration::from_secs(config.timeout);
        let connect_timeout = Duration::from_secs(config.connect_timeout);

        let client = ClientBuilder::new(
            reqwest::Client::builder()
                .timeout(timeout)
                .connect_timeout(connect_timeout)
                .user_agent(user_agent)
                .gzip(true)
                .build()?,
        )
        .with(LoggingMiddleware)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

        let host_credentials: HashMap<String, HostCredential> = credentials
            .into_iter()
            .map(|auth| (auth.host.clone(), auth))
            .collect();

        Ok(HttpClient {
            inner: client,
            host_credentials,
        })
    }

    /// Fetches `url`, returning the body and the response's `Content-Type`
    /// header (stripped of parameters like `; charset=`).
    pub async fn fetch(&self, url: &str) -> WyrmResult<(Bytes, Option<String>)> {
        let response = self.send(url, None).await?;
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.split(';').next().unwrap_or(v).trim().to_ascii_lowercase());

        Ok((Self::read_body(response).await?, content_type))
    }

    pub async fn fetch_conditional(
        &self,
        url: &str,
        validators: &HttpValidators,
    ) -> WyrmResult<FeedResponse> {
        let response = self.send(url, Some(validators)).await?;
        let read_header = |name| {
            response
                .headers()
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(String::from)
        };
        let etag = read_header(header::ETAG);
        let last_modified = read_header(header::LAST_MODIFIED);
        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(FeedResponse::NotModified(HttpValidators {
                etag: etag.or_else(|| validators.etag.clone()),
                last_modified: last_modified.or_else(|| validators.last_modified.clone()),
            }));
        }

        Ok(FeedResponse::Modified {
            body: Self::read_body(response).await?,
            validators: HttpValidators {
                etag,
                last_modified,
            },
        })
    }

    async fn send(&self, url: &str, validators: Option<&HttpValidators>) -> WyrmResult<Response> {
        let mut request = self.inner.get(url);
        if let Some(validators) = validators {
            if let Some(etag) = &validators.etag {
                request = request.header(header::IF_NONE_MATCH, etag);
            }
            if let Some(last_modified) = &validators.last_modified {
                request = request.header(header::IF_MODIFIED_SINCE, last_modified);
            }
        }

        let parsed_url = Url::parse(url).ok();
        let credential = parsed_url
            .as_ref()
            .and_then(host_key)
            .and_then(|host| self.host_credentials.get(&host));

        if let Some(credential) = credential {
            debug!(
                "sending {:?} credential for {}",
                credential.kind, credential.host
            );
            request = match credential.kind {
                HostCredentialKind::BasicAuth => {
                    request.basic_auth(&credential.name, Some(&credential.secret))
                }
                HostCredentialKind::CustomHeader => {
                    let (name, value) = custom_header(&credential.name, &credential.secret)?;
                    request.header(name, value)
                }
            }
        }

        let response = request.send().await?;

        // The status error alone can't tell a missing credential from a wrong one.
        let status = response.status();
        if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
            let final_url = response.url();
            let redirect_note = if parsed_url.as_ref() == Some(final_url) {
                String::new()
            } else {
                format!(" (redirected to {final_url})")
            };
            warn!(
                "{status} fetching {url}{redirect_note}; credential sent: {}",
                credential.is_some()
            );
        }

        Ok(response.error_for_status()?)
    }

    async fn read_body(mut response: Response) -> WyrmResult<Bytes> {
        if response
            .content_length()
            .is_some_and(|len| len as usize > MAX_RESPONSE_BYTES)
        {
            return Err(HttpClientError::ResponseTooLarge(MAX_RESPONSE_BYTES).into());
        }

        let mut body = BytesMut::new();
        while let Some(chunk) = response.chunk().await? {
            if body.len() + chunk.len() > MAX_RESPONSE_BYTES {
                return Err(HttpClientError::ResponseTooLarge(MAX_RESPONSE_BYTES).into());
            }
            body.extend_from_slice(&chunk);
        }

        Ok(body.freeze())
    }

    pub async fn post_json(&self, url: &str, body: &serde_json::Value) -> WyrmResult<()> {
        self.inner
            .post(url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body.to_string())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

/// Credential lookup key for a URL: `host:port`, lowercased, port defaulted.
pub fn host_key(url: &Url) -> Option<String> {
    Some(format!(
        "{}:{}",
        url.host_str()?.to_lowercase(),
        url.port_or_known_default()?
    ))
}

/// Accepts a full URL or a bare `host[:port]` (https assumed).
pub fn parse_host_key(input: &str) -> WyrmResult<String> {
    let input = input.trim();
    let with_scheme = if input.contains("://") {
        input.to_owned()
    } else {
        format!("https://{input}")
    };
    Url::parse(&with_scheme)
        .ok()
        .and_then(|url| host_key(&url))
        .ok_or_else(|| WyrmError::InvalidInput("invalid host".into()))
}

/// Header sent for a custom-header credential; also validates it on write.
pub fn custom_header(name: &str, value: &str) -> WyrmResult<(HeaderName, HeaderValue)> {
    let invalid = || WyrmError::InvalidInput("invalid header name or value".into());
    let header_name: HeaderName = name.parse().map_err(|_| invalid())?;
    let header_value: HeaderValue = value.parse().map_err(|_| invalid())?;

    Ok((header_name, header_value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host_key_of(url: &str) -> Option<String> {
        host_key(&Url::parse(url).unwrap())
    }

    #[test]
    fn host_key_lowercases_and_defaults_port_by_scheme() {
        assert_eq!(
            host_key_of("https://Feeds.Example.com/rss").as_deref(),
            Some("feeds.example.com:443")
        );
        // http and https keys differ, so credentials never match a plaintext request.
        assert_eq!(
            host_key_of("http://feeds.example.com/rss").as_deref(),
            Some("feeds.example.com:80")
        );
        assert_eq!(
            host_key_of("https://feeds.example.com:8443/").as_deref(),
            Some("feeds.example.com:8443")
        );
        assert_eq!(host_key_of("mailto:someone@example.com"), None);
    }

    #[test]
    fn parse_host_key_accepts_bare_hosts_and_full_urls() {
        assert_eq!(
            parse_host_key("feeds.example.com").unwrap(),
            "feeds.example.com:443"
        );
        assert_eq!(
            parse_host_key("  feeds.example.com  ").unwrap(),
            "feeds.example.com:443"
        );
        assert_eq!(
            parse_host_key("feeds.example.com:8080").unwrap(),
            "feeds.example.com:8080"
        );
        assert_eq!(
            parse_host_key("http://feeds.example.com/rss?x=1").unwrap(),
            "feeds.example.com:80"
        );
    }

    #[test]
    fn parse_host_key_rejects_invalid_input() {
        assert!(parse_host_key("").is_err());
        assert!(parse_host_key("not a host").is_err());
        assert!(parse_host_key("feeds.example.com:notaport").is_err());
    }

    #[test]
    fn custom_header_rejects_invalid_names_and_values() {
        assert!(custom_header("x-api-key", "secret").is_ok());
        assert!(custom_header("bad name", "secret").is_err());
        assert!(custom_header("", "secret").is_err());
        // A newline in the value would let a stored secret inject extra headers.
        assert!(custom_header("x-api-key", "secret\r\nx-injected: 1").is_err());
    }
}
