use bytes::Bytes;
use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next, Result};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use smart_default::SmartDefault;
use std::time::Duration;
use tracing::info;
use utils::{result::WyrmResult, settings::WyrmSettings};

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

#[derive(Clone)]
pub struct HttpClient {
    inner: ClientWithMiddleware,
}

#[derive(Debug, SmartDefault)]
pub struct HttpClientBuilder {
    timeout: Duration,
    connect_timeout: Duration,
    retries: u32,
    user_agent: String,
}

impl HttpClientBuilder {
    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    pub fn connect_timeout(mut self, secs: u64) -> Self {
        self.connect_timeout = Duration::from_secs(secs);
        self
    }

    pub fn retries(mut self, max: u32) -> Self {
        self.retries = max;
        self
    }

    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = ua.into();
        self
    }

    pub fn build(&self) -> WyrmResult<HttpClient> {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(self.retries);

        let client = ClientBuilder::new(
            reqwest::Client::builder()
                .timeout(self.timeout)
                .connect_timeout(self.connect_timeout)
                .user_agent(self.user_agent.clone())
                .gzip(true)
                .build()?,
        )
        .with(LoggingMiddleware)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

        Ok(HttpClient { inner: client })
    }
}

impl HttpClient {
    pub fn builder(settings: &WyrmSettings) -> HttpClientBuilder {
        let mut builder = HttpClientBuilder::default();

        builder = builder.timeout(settings.http.timeout);
        builder = builder.connect_timeout(settings.http.connect_timeout);
        builder = builder.retries(settings.http.retries);
        builder = builder.user_agent(settings.http.user_agent.clone());

        builder
    }

    pub async fn fetch(&self, url: &str) -> WyrmResult<Bytes> {
        let response = self.inner.get(url).send().await?;
        let bytes = response.bytes().await?;

        Ok(bytes)
    }
}
