use crate::{
    DatabasePool,
    models::folder::Folder,
    newtypes::{FeedId, FolderId},
    schema::feeds,
};
use chrono::{DateTime, Utc};
use diesel::{
    Selectable,
    prelude::{Queryable, *},
};
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::Serialize;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[serde_with::skip_serializing_none]
#[derive(Clone, Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::feeds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(ts_rs::TS)]
#[ts(optional_fields, export)]
pub struct Feed {
    pub id: FeedId,
    /// Human-readable feed name shown in the UI.
    pub title: String,
    /// URL the feed is fetched from.
    pub url: String,
    /// Refresh interval in minutes; the feed is polled once this much time has
    /// elapsed since `last_fetched_at`.
    pub ttl: i32,
    /// Substrings that exclude a post when matched against its URL.
    pub filters: Vec<Option<String>>,
    /// Timestamp of the last successful poll; `None` until the feed is first fetched.
    pub last_fetched_at: Option<DateTime<Utc>>,
    /// Timestamp the feed was added.
    pub created_at: DateTime<Utc>,
    /// Optional folder used to group and filter feeds.
    pub folder_id: Option<FolderId>,
}

impl Feed {
    pub async fn get(pool: &DatabasePool, feed_id: FeedId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        feeds::table
            .find(feed_id)
            .select(Feed::as_select())
            .first(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn get_all(pool: &DatabasePool) -> WyrmResult<Vec<Self>> {
        let mut conn = pool.get().await?;
        feeds::table
            .select(Self::as_select())
            .load(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn create(
        pool: &DatabasePool,
        mut form: FeedInsertForm,
        folder: Option<&str>,
    ) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        let conn = &mut *conn;

        conn.transaction(async |conn| {
            // Frontend sends a folder name. We try to resolve that name to a folder id (folder
            // already exists) or create a new folder to put the feed in.
            if let Some(name) = folder.map(str::trim).filter(|n| !n.is_empty()) {
                form.folder_id = Some(Folder::resolve_or_create_on(conn, name).await?.id);
            }

            let feed = diesel::insert_into(feeds::table)
                .values(form)
                .get_result::<Self>(conn)
                .await?;

            Ok::<Feed, WyrmError>(feed)
        })
        .await
    }

    pub async fn update(
        pool: &DatabasePool,
        mut form: FeedUpdateForm,
        folder: Option<Option<&str>>,
    ) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        let conn = &mut *conn;

        conn.transaction(async |conn| {
            if let Some(folder) = folder {
                form.folder_id = match folder.map(str::trim).filter(|n| !n.is_empty()) {
                    Some(name) => Some(Some(Folder::resolve_or_create_on(conn, name).await?.id)),
                    None => Some(None),
                };
            }

            let feed = diesel::update(feeds::table.find(form.id))
                .set(form)
                .get_result::<Self>(conn)
                .await?;

            Ok::<Feed, WyrmError>(feed)
        })
        .await
    }

    pub async fn delete(pool: &DatabasePool, feed_id: FeedId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::delete(feeds::table.find(feed_id))
            .returning(Self::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub fn is_due(&self) -> bool {
        match self.last_fetched_at {
            None => true,
            Some(last_fetched_at) => {
                let elapsed = Utc::now() - last_fetched_at;
                elapsed.num_minutes() >= self.ttl as i64
            }
        }
    }
}

#[derive(Default, Insertable)]
#[diesel(table_name = crate::schema::feeds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FeedInsertForm {
    pub title: String,
    pub url: String,
    pub ttl: i32,
    pub folder_id: Option<FolderId>,
    pub filters: Option<Vec<Option<String>>>,
}

#[derive(Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::feeds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FeedUpdateForm {
    pub id: FeedId,
    pub title: Option<String>,
    pub url: Option<String>,
    pub ttl: Option<i32>,
    /// Double `Option` for the nullable column:
    /// `None` leaves the folder untouched (e.g. the worker bumping `last_fetched_at`);
    /// `Some(None)` clears it;
    /// `Some(Some(id))` assigns/updates it.
    pub folder_id: Option<Option<FolderId>>,
    pub filters: Option<Vec<Option<String>>>,
    pub last_fetched_at: Option<DateTime<Utc>>,
}

/// Test helper: inserts a feed with a unique url and returns the `Feed`. Lives
/// here (with the `Feed` model) and is shared with the post tests via
/// `#[macro_use]` on this module in `models/mod.rs`. Delete the feed to clean
/// up (the cascade removes any posts attached to it).
#[cfg(test)]
macro_rules! test_feed {
    ($pool:expr) => {
        $crate::models::feed::Feed::create(
            $pool,
            $crate::models::feed::FeedInsertForm {
                title: "test feed".to_string(),
                url: format!(
                    "https://example.com/feed/{}",
                    chrono::Utc::now()
                        .timestamp_nanos_opt()
                        .expect("timestamp in range")
                ),
                ttl: 60,
                ..Default::default()
            },
            None,
        )
        .await
        .expect("should create test feed")
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::folder::Folder, setup_test_db};

    #[tokio::test]
    async fn create_then_get_roundtrips() {
        let pool = setup_test_db().await;
        let created = test_feed!(&pool);

        let got = Feed::get(&pool, created.id)
            .await
            .expect("get should succeed");
        assert_eq!(got.id, created.id);
        assert_eq!(got.url, created.url);
        assert_eq!(got.ttl, created.ttl);

        Feed::delete(&pool, created.id)
            .await
            .expect("should delete feed");
    }

    #[tokio::test]
    async fn update_changes_given_fields() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);
        // The folder_id FK needs a real row — the shared test db has none.
        let folder = Folder::resolve_or_create(&pool, "feed update test")
            .await
            .expect("should create folder");

        let updated = Feed::update(
            &pool,
            FeedUpdateForm {
                id: feed.id,
                title: Some("Renamed".to_string()),
                url: None,
                ttl: Some(120),
                folder_id: Some(Some(folder.id)),
                filters: None,
                last_fetched_at: None,
            },
            None,
        )
        .await
        .expect("update should succeed");

        assert_eq!(updated.title, "Renamed");
        assert_eq!(updated.ttl, 120);
        assert_eq!(updated.folder_id, Some(folder.id));
        // A `None` field is left unchanged.
        assert_eq!(updated.url, feed.url);

        Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
        Folder::delete(&pool, folder.id)
            .await
            .expect("should delete folder");
    }

    #[tokio::test]
    async fn delete_returns_and_removes_feed() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);

        let deleted = Feed::delete(&pool, feed.id)
            .await
            .expect("delete should succeed");
        assert_eq!(deleted.id, feed.id);
        assert!(Feed::get(&pool, feed.id).await.is_err());
    }

    #[tokio::test]
    async fn get_all_includes_created_feed() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);

        let all = Feed::get_all(&pool).await.expect("get_all should succeed");
        assert!(all.iter().any(|f| f.id == feed.id));

        Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
    }
}
