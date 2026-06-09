use database::{DatabasePool, utils::settings::RuntimeSettings};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::Sender;
use wyrm_rss::{http::HttpClient, worker::WorkerCommand};

#[derive(Clone)]
pub struct WyrmContext {
    /// Async database connection pool.
    pub db_pool: DatabasePool,
    /// Runtime settings loaded from the database. Shared between RSS worker
    /// and actix handlers threads so that setting updates take effect
    /// immediately without a restart.
    pub runtime_settings: Arc<RwLock<RuntimeSettings>>,
    /// HTTP client used for fetching RSS feeds.
    pub http: HttpClient,
    /// Channel to send commands to the background RSS worker.
    pub worker_tx: Sender<WorkerCommand>,
}
