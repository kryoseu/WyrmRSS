use database::DatabasePool;
use utils::settings::WyrmSettings;
use wyrm_rss::http::HttpClient;

#[derive(Clone)]
pub struct WyrmContext {
    pub db_pool: DatabasePool,
    pub settings: WyrmSettings,
    pub http: HttpClient,
}
