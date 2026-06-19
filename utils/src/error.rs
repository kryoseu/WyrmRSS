use actix_web::{
    HttpResponse,
    error,
    http::{StatusCode, header::ContentType},
};
use diesel::result::DatabaseErrorKind;

#[derive(thiserror::Error, Debug)]
pub enum HttpClientError {
    #[error("http error: {0}")]
    ClientError(#[from] reqwest::Error),
    #[error("middleware error: {0}")]
    MiddlewareError(#[from] reqwest_middleware::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum HttpServerError {
    #[error("bind error: {0}")]
    BindError(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("pool error: {0}")]
    PoolError(deadpool::managed::PoolError<diesel_async::pooled_connection::PoolError>),
    #[error("pool builder error: {0}")]
    PoolBuildError(deadpool::managed::BuildError),
    #[error("connection error: {0}")]
    ConnectionError(diesel::result::ConnectionError),
    #[error("migration error: {0}")]
    MigrationError(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("query failed: {0}")]
    QueryFailed(diesel::result::Error),
    #[error("unique violation: {0}")]
    UniqueViolation(diesel::result::Error),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("not found")]
    NotFound,
    #[error("connection timeout")]
    Timeout,
}

#[derive(thiserror::Error, Debug)]
pub enum WyrmError {
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error(transparent)]
    HttpServer(#[from] HttpServerError),
    #[error(transparent)]
    HttpClient(#[from] HttpClientError),
    #[error("rss parse feed error: {0}")]
    ParseFeedError(#[from] feed_rs::parser::ParseFeedError),
    #[error("xml serialize error: {0}")]
    XmlSerializeError(quick_xml::se::SeError),
    #[error("xml deserialize error: {0}")]
    XmlDeserializeError(quick_xml::de::DeError),
    #[error("rss worker error: {0}")]
    WorkerError(String),
    #[error("lock poisoned: {0}")]
    LockPoisoned(String),
    #[error("invalid webhook template: {0}")]
    WebhookTemplate(String),
    #[error("invalid config: {0}")]
    StartupConfigError(#[from] config::ConfigError),
}

impl From<reqwest::Error> for WyrmError {
    fn from(e: reqwest::Error) -> Self {
        WyrmError::HttpClient(HttpClientError::ClientError(e))
    }
}

impl From<reqwest_middleware::Error> for WyrmError {
    fn from(e: reqwest_middleware::Error) -> Self {
        WyrmError::HttpClient(HttpClientError::MiddlewareError(e))
    }
}

impl From<deadpool::managed::PoolError<diesel_async::pooled_connection::PoolError>> for WyrmError {
    fn from(e: deadpool::managed::PoolError<diesel_async::pooled_connection::PoolError>) -> Self {
        WyrmError::Database(DatabaseError::PoolError(e))
    }
}

impl From<diesel::result::ConnectionError> for WyrmError {
    fn from(e: diesel::result::ConnectionError) -> Self {
        WyrmError::Database(DatabaseError::ConnectionError(e))
    }
}
impl<T> From<std::sync::PoisonError<T>> for WyrmError {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        WyrmError::LockPoisoned(e.to_string())
    }
}

impl From<diesel::result::Error> for WyrmError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => WyrmError::Database(DatabaseError::NotFound),
            diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                WyrmError::Database(DatabaseError::UniqueViolation(e))
            }
            _ => WyrmError::Database(DatabaseError::QueryFailed(e)),
        }
    }
}

impl WyrmError {
    /// A safe, client-facing message. Server faults collapse to a generic
    /// string so internal diesel/pool detail never reaches the response body;
    /// client errors expose only curated text.
    fn client_message(&self) -> &str {
        match self {
            WyrmError::Database(DatabaseError::NotFound) => "not found",
            WyrmError::Database(DatabaseError::Conflict(msg)) => msg,
            WyrmError::Database(DatabaseError::UniqueViolation(_)) => "conflict",
            WyrmError::XmlDeserializeError(_) => "invalid request body",
            WyrmError::WebhookTemplate(msg) => msg,
            _ => "internal server error",
        }
    }
}

impl error::ResponseError for WyrmError {
    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        // Log the full internal error for server faults; client errors
        // (404/409/400) are expected, so keep them at debug to avoid noise.
        if status.is_server_error() {
            tracing::error!("{self}");
        } else {
            tracing::debug!("{self}");
        }
        let body = serde_json::json!({ "error": self.client_message() }).to_string();
        HttpResponse::build(status)
            .insert_header(ContentType::json())
            .body(body)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            WyrmError::Database(DatabaseError::NotFound) => StatusCode::NOT_FOUND,
            WyrmError::Database(DatabaseError::UniqueViolation(_)) => StatusCode::CONFLICT,
            WyrmError::Database(DatabaseError::Conflict(_)) => StatusCode::CONFLICT,
            WyrmError::XmlDeserializeError(_) => StatusCode::BAD_REQUEST,
            WyrmError::WebhookTemplate(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
