use database::DatabasePool;
use tokio::sync::mpsc::Sender;
use wyrm_rss::{http::HttpClient, worker::WorkerCommand};
use wyrm_utils::settings::WyrmSettings;

#[derive(Clone)]
pub struct WyrmContext {
    pub db_pool: DatabasePool,
    pub settings: WyrmSettings,
    pub http: HttpClient,
    pub worker_tx: Sender<WorkerCommand>,
}
