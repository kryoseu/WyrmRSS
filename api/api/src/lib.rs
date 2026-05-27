use chrono::{DateTime, Utc};
use serde::Deserialize;

pub mod feeds;
pub mod posts;

#[derive(Deserialize)]
pub struct CursorQuery {
    pub timestamp: Option<i64>,
    pub post_id: Option<i32>,
}

impl CursorQuery {
    pub fn to_cursor(&self) -> Option<(DateTime<Utc>, i32)> {
        match (self.timestamp, self.post_id) {
            (Some(ms), Some(id)) => Some((
                DateTime::from_timestamp_millis(ms).unwrap_or_else(Utc::now),
                id,
            )),
            _ => None,
        }
    }
}
