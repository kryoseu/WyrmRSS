use crate::models::settings::Settings;

pub struct RuntimeSettings {
    pub page_size: i32,
    pub feed_poll_interval_secs: i32,
    pub http_timeout: i32,
    pub http_connect_timeout: i32,
    pub http_retries: i32,
    pub http_user_agent: Option<String>,
    pub expire_read_after_days: Option<i32>,
    pub expire_unread_after_days: Option<i32>,
}

impl From<&Settings> for RuntimeSettings {
    fn from(s: &Settings) -> Self {
        RuntimeSettings {
            page_size: s.page_size,
            feed_poll_interval_secs: s.feed_poll_interval_secs,
            http_timeout: s.http_timeout,
            http_connect_timeout: s.http_connect_timeout,
            http_retries: s.http_retries,
            http_user_agent: s.http_user_agent.clone(),
            expire_read_after_days: s.expire_read_after_days,
            expire_unread_after_days: s.expire_unread_after_days,
        }
    }
}
