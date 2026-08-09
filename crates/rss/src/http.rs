use bytes::{Bytes, BytesMut};
use database::utils::settings::RuntimeSettings;
use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next, Result};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use std::time::Duration;
use tracing::info;
use wyrm_utils::{error::HttpClientError, result::WyrmResult};

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
}

impl HttpClient {
    pub fn new(config: &HttpConfig) -> WyrmResult<HttpClient> {
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

        Ok(HttpClient { inner: client })
    }

    /// Fetches `url`, returning the body and the response's `Content-Type`
    /// header (stripped of parameters like `; charset=`).
    pub async fn fetch(&self, url: &str) -> WyrmResult<(Bytes, Option<String>)> {
        let mut response = self.inner.get(url).send().await?;

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
