use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PaginationCursor(pub String);

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PagedResponse<T: ts_rs::TS> {
    pub items: T,
    #[ts(optional)]
    pub next_page: Option<PaginationCursor>,
}

impl PaginationCursor {
    pub fn encode(published_at: DateTime<Utc>, id: i32) -> Self {
        Self(format!("{}:{}", published_at.timestamp_millis(), id))
    }

    pub fn decode(&self) -> Option<(DateTime<Utc>, i32)> {
        let (ms_str, id_str) = self.0.split_once(':')?;
        let ms: i64 = ms_str.parse().ok()?;
        let id: i32 = id_str.parse().ok()?;
        Some((DateTime::from_timestamp_millis(ms)?, id))
    }
}
