#[derive(Debug, thiserror::Error)]
pub enum WyrmDesktopError {
    #[error("failed to resolve app data directory: {0}")]
    AppDataDir(String),
    #[error("embedded postgres error: {0}")]
    Postgres(#[from] postgresql_embedded::Error),
    #[error(transparent)]
    Backend(#[from] wyrm_utils::error::WyrmError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type WyrmDesktopResult<T> = Result<T, WyrmDesktopError>;
