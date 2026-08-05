use crate::{
    DatabasePool,
    models::folder::Folder,
    newtypes::{FeedId, Filters, FolderId},
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
#[diesel(check_for_backend(crate::Backend))]
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
    pub filters: Filters,
    /// Timestamp of the last successful poll; `None` until the feed is first fetched.
    pub last_fetched_at: Option<DateTime<Utc>>,
    /// Timestamp the feed was added.
    pub created_at: DateTime<Utc>,
    /// Optional folder used to group and filter feeds.
    pub folder_id: Option<FolderId>,
    /// Whether the feed is paused (not polled by the worker).
    pub is_paused: bool,
}

impl Feed {
    pub async fn get(pool: &DatabasePool, id: FeedId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        feeds::table
            .find(id)
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

    pub async fn create(pool: &DatabasePool, mut form: FeedInsertForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        let conn = &mut *conn;

        conn.transaction(async |conn| {
            // Folder name resolved to a FolderId.
            // Resolved inside the transaction so a failed feed insert can't
            // leave behind a newly created folder.
            let folder_id = Folder::resolve_name_on(conn, form.folder.take().as_deref()).await?;

            let feed = diesel::insert_into(feeds::table)
                .values((form, feeds::folder_id.eq(folder_id)))
                .get_result::<Self>(conn)
                .await?;

            Ok::<Feed, WyrmError>(feed)
        })
        .await
    }

    pub async fn update(pool: &DatabasePool, mut form: FeedUpdateForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        let conn = &mut *conn;

        conn.transaction(async |conn| {
            // Folder name resolved to a FolderId.
            let folder_id = match form.folder.take() {
                None => None,
                Some(folder) => Some(Folder::resolve_name_on(conn, folder.as_deref()).await?),
            };

            let feed = diesel::update(feeds::table.find(form.id))
                .set((form, folder_id.map(|f| feeds::folder_id.eq(f))))
                .get_result::<Self>(conn)
                .await?;

            Ok::<Feed, WyrmError>(feed)
        })
        .await
    }

    pub async fn delete(pool: &DatabasePool, id: FeedId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::delete(feeds::table.find(id))
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
#[diesel(check_for_backend(crate::Backend))]
pub struct FeedInsertForm {
    pub title: String,
    pub url: String,
    pub ttl: i32,
    /// Name of the folder to place the feed in;
    /// Skips insertion here since callers (frontend) always sends
    /// a folder name, however feed insertion requires a FolderId.
    /// This resolution folder name <> FolderId happens inside the
    /// create transaction.
    #[diesel(skip_insertion)]
    pub folder: Option<String>,
    pub filters: Option<Filters>,
}

#[derive(Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::feeds)]
#[diesel(check_for_backend(crate::Backend))]
pub struct FeedUpdateForm {
    pub id: FeedId,
    pub title: Option<String>,
    pub url: Option<String>,
    pub ttl: Option<i32>,
    /// Double `Option` for the nullable column:
    /// `None` leaves the folder untouched (e.g. the worker bumping `last_fetched_at`);
    /// `Some(None)` or a blank name clears it;
    /// `Some(Some(name))` assigns it, resolved inside the update transaction
    /// (creating the folder if the name is new).
    /// Skips the update for the same reason [`FeedInsertForm::folder`] skips
    /// insertion: this is a name, not the `folder_id` the column needs.
    #[diesel(skip_update)]
    pub folder: Option<Option<String>>,
    pub filters: Option<Filters>,
    pub is_paused: Option<bool>,
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
        // Pre-created so the name below resolves to a known id we can assert
        // against and clean up.
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
                folder: Some(Some(folder.name.clone())),
                filters: None,
                is_paused: Some(true),
                last_fetched_at: None,
            },
        )
        .await
        .expect("update should succeed");

        assert_eq!(updated.title, "Renamed");
        assert_eq!(updated.ttl, 120);
        assert_eq!(updated.folder_id, Some(folder.id));
        assert!(updated.is_paused);
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
