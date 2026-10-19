use bytes::{Bytes, BytesMut};
use database::{
    models::host_credential::{HostCredential, HostCredentialKind},
    utils::settings::RuntimeSettings,
};
use http::{Extensions, HeaderName, HeaderValue};
use reqwest::{Request, Response, StatusCode, Url};
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
        let mut request = self.inner.get(url);

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

        let mut response = response.error_for_status()?;

        if response
            .content_length()
            .is_some_and(|len| len as usize > MAX_RESPONSE_BYTES)
        {
            return Err(HttpClientError::ResponseTooLarge(MAX_RESPONSE_BYTES).into());
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.split(';').next().unwrap_or(v).trim().to_ascii_lowercase());

        let mut body = BytesMut::new();
        while let Some(chunk) = response.chunk().await? {
            if body.len() + chunk.len() > MAX_RESPONSE_BYTES {
                return Err(HttpClientError::ResponseTooLarge(MAX_RESPONSE_BYTES).into());
            }
            body.extend_from_slice(&chunk);
        }

        Ok((body.freeze(), content_type))
    }

    pub async fn post_json(&self, url: &str, body: &serde_json::Value) -> WyrmResult<()> {
        self.inner
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
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
