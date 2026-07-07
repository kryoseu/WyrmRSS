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
    /// `timestamp` is the view's leading sort column (`created_at` for posts,
    /// `archived_at` for the archive); `id` breaks ties within it.
    ///
    /// Encoded at microsecond precision to round-trip TIMESTAMPTZ exactly:
    /// a truncated timestamp would fail the keyset filter's equality branch
    /// and skip rows that share the boundary row's timestamp.
    pub fn encode(timestamp: DateTime<Utc>, id: i32) -> Self {
        Self(format!("{}:{}", timestamp.timestamp_micros(), id))
    }

    pub fn decode(&self) -> Option<(DateTime<Utc>, i32)> {
        let (us_str, id_str) = self.0.split_once(':')?;
        let us: i64 = us_str.parse().ok()?;
        let id: i32 = id_str.parse().ok()?;
        Some((DateTime::from_timestamp_micros(us)?, id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    #[test]
    fn cursor_roundtrips_microsecond_timestamps() {
        // Sub-millisecond remainder, as produced by Postgres NOW().
        let timestamp = "2026-07-05T23:50:29.357711Z"
            .parse::<DateTime<Utc>>()
            .unwrap();

        let decoded = PaginationCursor::encode(timestamp, 42).decode();

        assert_eq!(decoded, Some((timestamp, 42)));
    }

    #[test]
    fn decode_rejects_malformed_cursors() {
        for raw in ["", "123", "abc:1", "123:xyz", ":"] {
            assert_eq!(PaginationCursor(raw.to_string()).decode(), None);
        }
    }
}
