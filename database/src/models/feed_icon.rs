use crate::{DatabasePool, newtypes::FeedId, schema::feed_icons};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, upsert::excluded};
use diesel_async::RunQueryDsl;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

/// A feed's resolved icon. One row per feed; `data` is `None` when a lookup
/// completed but found nothing, so missing icons can be retried on a slow
/// cadence instead of on every poll.
#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::feed_icons)]
#[diesel(primary_key(feed_id))]
#[diesel(check_for_backend(crate::Backend))]
pub struct FeedIcon {
    pub feed_id: FeedId,
    /// Raw image bytes; `None` = checked, nothing found.
    pub data: Option<Vec<u8>>,
    /// MIME type the bytes are served with.
    pub content_type: Option<String>,
    /// When the icon was last (re)resolved.
    pub checked_at: DateTime<Utc>,
}

impl FeedIcon {
    /// `None` = the feed has never been checked for an icon.
    pub async fn get(pool: &DatabasePool, feed_id: FeedId) -> WyrmResult<Option<Self>> {
        let mut conn = pool.get().await?;
        feed_icons::table
            .find(feed_id)
            .select(Self::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(WyrmError::from)
    }

    /// Records a lookup result. Resolution always writes the latest outcome,
    /// so an existing row is replaced (`checked_at` bumps either way).
    pub async fn create(pool: &DatabasePool, form: FeedIconInsertForm) -> WyrmResult<()> {
        let mut conn = pool.get().await?;
        diesel::insert_into(feed_icons::table)
            .values(&form)
            .on_conflict(feed_icons::feed_id)
            .do_update()
            .set((
                feed_icons::data.eq(excluded(feed_icons::data)),
                feed_icons::content_type.eq(excluded(feed_icons::content_type)),
                feed_icons::checked_at.eq(diesel::dsl::now),
            ))
            .execute(&mut conn)
            .await
            .map_err(WyrmError::from)?;
        Ok(())
    }

    /// Drops the stored icon so the next resolution starts fresh; used when a
    /// feed's URL changes. Deleting a feed cascades the row away on its own.
    pub async fn delete(pool: &DatabasePool, feed_id: FeedId) -> WyrmResult<()> {
        let mut conn = pool.get().await?;
        diesel::delete(feed_icons::table.find(feed_id))
            .execute(&mut conn)
            .await
            .map_err(WyrmError::from)?;
        Ok(())
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::feed_icons)]
#[diesel(check_for_backend(crate::Backend))]
pub struct FeedIconInsertForm {
    pub feed_id: FeedId,
    /// `None` records a lookup that found no icon.
    pub data: Option<Vec<u8>>,
    pub content_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup_test_db;

    #[tokio::test]
    async fn create_replaces_and_get_roundtrips() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);

        assert!(FeedIcon::get(&pool, feed.id).await.unwrap().is_none());

        FeedIcon::create(
            &pool,
            FeedIconInsertForm {
                feed_id: feed.id,
                data: Some(vec![1, 2, 3]),
                content_type: Some("image/png".into()),
            },
        )
        .await
        .expect("create should succeed");
        let icon = FeedIcon::get(&pool, feed.id)
            .await
            .unwrap()
            .expect("icon row should exist");
        assert_eq!(icon.data.as_deref(), Some(&[1u8, 2, 3][..]));
        assert_eq!(icon.content_type.as_deref(), Some("image/png"));

        // A later "nothing found" result replaces the row instead of erroring
        // on the primary key.
        FeedIcon::create(
            &pool,
            FeedIconInsertForm {
                feed_id: feed.id,
                data: None,
                content_type: None,
            },
        )
        .await
        .expect("create should succeed");
        let icon = FeedIcon::get(&pool, feed.id)
            .await
            .unwrap()
            .expect("icon row should exist");
        assert!(icon.data.is_none());
        assert!(icon.content_type.is_none());

        crate::models::feed::Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
    }
}
