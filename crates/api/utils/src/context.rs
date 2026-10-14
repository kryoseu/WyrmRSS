use database::{DatabasePool, utils::settings::RuntimeSettings};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::Sender;
use wyrm_rss::{http::HttpClient, worker::WorkerCommand};
use wyrm_utils::result::WyrmResult;

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
    /// Whether or not this server is running in demo mode.
    /// Demo mode means certain APIs are restricted.
    /// Used for https://demo.wyrmrss.com
    pub run_as_demo: bool,
}

impl WyrmContext {
    pub fn deny_in_demo(&self) -> WyrmResult<()> {
        if self.run_as_demo {
            return Err(wyrm_utils::error::WyrmError::DemoRestricted);
        }
        Ok(())
    }
}
