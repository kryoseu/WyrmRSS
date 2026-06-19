use bytes::Bytes;
use database::utils::settings::RuntimeSettings;
use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next, Result};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use std::time::Duration;
use tracing::info;
use wyrm_utils::result::WyrmResult;

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

    pub async fn fetch(&self, url: &str) -> WyrmResult<Bytes> {
        let response = self.inner.get(url).send().await?;
        let bytes = response.bytes().await?;

        Ok(bytes)
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
