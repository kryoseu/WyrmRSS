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
    pub fn encode(id: i32) -> Self {
        Self(id.to_string())
    }

    pub fn decode(&self) -> Option<i32> {
        self.0.parse().ok()
    }
}
