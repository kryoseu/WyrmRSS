use crate::{
    DatabasePool,
    models::feed::Feed,
    newtypes::FeedId,
    schema::{feed_poll_state, feeds},
};
use chrono::{DateTime, TimeDelta, Utc};
use diesel::{prelude::*, upsert::excluded};
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::Serialize;
use std::collections::HashMap;
use wyrm_utils::result::WyrmResult;

/// Internal poll metadata. Validators belong to source_url, never to another feed URL.
#[derive(Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = feed_poll_state)]
#[diesel(check_for_backend(crate::Backend))]
pub struct FeedPollState {
    pub feed_id: FeedId,
    pub source_url: String,
    pub filter_signature: String,
    pub checked_at: DateTime<Utc>,
    pub last_error: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, ts_rs::TS)]
#[ts(optional_fields, export)]
pub struct FeedHealth {
    pub last_checked_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    /// Future eligibility time; absent when already due or paused. Checks run on the global
    /// polling interval.
    pub next_check_at: Option<DateTime<Utc>>,
}

impl FeedHealth {
    pub fn for_feed(feed: &Feed, state: Option<&FeedPollState>) -> Self {
        let state = state.filter(|s| s.source_url == feed.url);
        Self {
            last_checked_at: state.map(|s| s.checked_at),
            last_success_at: feed.last_fetched_at,
            last_error: state.and_then(|s| s.last_error.clone()),
            next_check_at: if feed.is_paused {
                None
            } else {
                feed.last_fetched_at
                    .and_then(|t| t.checked_add_signed(TimeDelta::minutes(i64::from(feed.ttl))))
                    .filter(|time| *time > Utc::now())
            },
        }
    }
}

impl FeedPollState {
    pub fn matches(&self, feed: &Feed) -> bool {
        self.source_url == feed.url && self.filter_signature == Self::filters_key(feed)
    }

    pub fn filters_key(feed: &Feed) -> String {
        serde_json::to_string(&feed.filters).expect("feed filters serialize as a string array")
    }

    pub async fn all_by_feed(pool: &DatabasePool) -> WyrmResult<HashMap<FeedId, Self>> {
        let mut conn = pool.get().await?;
        let states = feed_poll_state::table
            .select(Self::as_select())
            .load::<Self>(&mut conn)
            .await?;
        Ok(states.into_iter().map(|s| (s.feed_id, s)).collect())
    }

    /// Lock the matching source row before saving. An old in-flight request must
    /// not overwrite health/validators after the user edits or deletes its feed.
    pub async fn record(&self, pool: &DatabasePool, success: bool) -> WyrmResult<()> {
        let mut conn = pool.get().await?;
        conn.transaction(async |conn| {
            let source = feeds::table
                .find(self.feed_id)
                .filter(feeds::url.eq(&self.source_url));
            let count = if success {
                diesel::update(source)
                    .set(feeds::last_fetched_at.eq(Some(self.checked_at)))
                    .execute(conn)
                    .await?
            } else {
                diesel::update(source)
                    .set(feeds::url.eq(&self.source_url))
                    .execute(conn)
                    .await?
            };
            if count == 0 {
                return Ok(());
            }
            diesel::insert_into(feed_poll_state::table)
                .values(self)
                .on_conflict(feed_poll_state::feed_id)
                .do_update()
                .set((
                    feed_poll_state::source_url.eq(excluded(feed_poll_state::source_url)),
                    feed_poll_state::filter_signature
                        .eq(excluded(feed_poll_state::filter_signature)),
                    feed_poll_state::checked_at.eq(excluded(feed_poll_state::checked_at)),
                    feed_poll_state::last_error.eq(excluded(feed_poll_state::last_error)),
                    feed_poll_state::etag.eq(excluded(feed_poll_state::etag)),
                    feed_poll_state::last_modified.eq(excluded(feed_poll_state::last_modified)),
                ))
                .execute(conn)
                .await?;
            Ok(())
        })
        .await
    }
}

#[cfg(all(test, feature = "postgres"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn postgres_poll_state_recovers_and_cascades() {
        let pool = crate::setup_test_db().await;
        let feed = test_feed!(&pool);
        let mut state = FeedPollState {
            feed_id: feed.id,
            source_url: feed.url.clone(),
            filter_signature: FeedPollState::filters_key(&feed),
            checked_at: Utc::now(),
            last_error: None,
            etag: Some("\"v1\"".into()),
            last_modified: None,
        };
        state.record(&pool, true).await.unwrap();
        state.last_error = Some("HTTP 503 Service Unavailable".into());
        state.record(&pool, false).await.unwrap();
        let stored = FeedPollState::all_by_feed(&pool)
            .await
            .unwrap()
            .remove(&feed.id)
            .unwrap();
        state.last_error = None;
        state.record(&pool, true).await.unwrap();
        let recovered = FeedPollState::all_by_feed(&pool)
            .await
            .unwrap()
            .remove(&feed.id)
            .unwrap();
        Feed::delete(&pool, feed.id).await.unwrap();
        assert!(stored.last_error.is_some());
        assert_eq!(stored.etag, state.etag);
        assert!(recovered.last_error.is_none());
        assert!(
            !FeedPollState::all_by_feed(&pool)
                .await
                .unwrap()
                .contains_key(&feed.id)
        );
    }
}
