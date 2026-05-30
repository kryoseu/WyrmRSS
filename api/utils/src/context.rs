use database::DatabasePool;
use tokio::sync::mpsc::Sender;
use utils::settings::WyrmSettings;
use wyrm_rss::{http::HttpClient, worker::FeedCommand};

#[derive(Clone)]
pub struct WyrmContext {
    pub db_pool: DatabasePool,
    pub settings: WyrmSettings,
    pub http: HttpClient,
    pub worker_tx: Sender<FeedCommand>,
}
