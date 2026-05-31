use actix_web::{
    HttpResponse,
    error,
    http::{StatusCode, header::ContentType},
};
use diesel::result::DatabaseErrorKind;

#[derive(thiserror::Error, Debug)]
pub enum HttpClientError {
    #[error("bind error: {0}")]
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
    #[error("rss worker error")]
    WorkerError,
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

impl error::ResponseError for WyrmError {
    fn error_response(&self) -> HttpResponse {
        tracing::error!("{}", self);
        let body = serde_json::json!({ "error": self.to_string() }).to_string();
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(body)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            WyrmError::Database(DatabaseError::NotFound) => StatusCode::NOT_FOUND,
            WyrmError::Database(DatabaseError::UniqueViolation(_)) => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
