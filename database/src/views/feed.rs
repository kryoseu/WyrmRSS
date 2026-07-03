use crate::{
    DatabasePool,
    models::feed::Feed,
    schema::{feeds, posts},
};
use diesel::{dsl::count, prelude::*};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FeedView {
    #[serde(flatten)]
    pub feed: Feed,
    pub unread_count: i64,
}

pub async fn list(pool: &DatabasePool) -> WyrmResult<Vec<FeedView>> {
    let mut conn = pool.get().await?;
    // The is_read filter lives in the join's ON clause, not a WHERE: filtering
    // after a left join would drop feeds whose posts are all read, while here
    // they survive with no joined rows and count to 0.
    let rows: Vec<(Feed, i64)> = feeds::table
        .left_join(posts::table.on(posts::feed_id.eq(feeds::id).and(posts::is_read.eq(false))))
        .group_by(feeds::id)
        .select((Feed::as_select(), count(posts::id.nullable())))
        .load(&mut conn)
        .await
        .map_err(WyrmError::from)?;

    Ok(rows
        .into_iter()
        .map(|(feed, unread_count)| FeedView { feed, unread_count })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::post::{Post, PostUpdateForm},
        newtypes::FeedId,
        setup_test_db,
    };

    /// The test database is shared across concurrently running tests, so the
    /// returned list can contain feeds from other tests — assert only on the
    /// feeds this test created.
    fn find(views: &[FeedView], id: FeedId) -> &FeedView {
        views
            .iter()
            .find(|v| v.feed.id == id)
            .expect("feed should be listed")
    }

    #[tokio::test]
    async fn list_counts_unread_per_feed_and_keeps_postless_feeds() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);
        let postless_feed = test_feed!(&pool);

        let _ = test_post!(&pool, feed.id);
        let _ = test_post!(&pool, feed.id);
        let read_id = test_post!(&pool, feed.id).id;
        Post::update(
            &pool,
            PostUpdateForm {
                id: read_id,
                is_favorite: None,
                is_read: Some(true),
            },
        )
        .await
        .expect("should mark post read");

        let views = list(&pool).await.expect("list should succeed");
        assert_eq!(find(&views, feed.id).unread_count, 2);
        // A feed with no unread rows must survive the left join with a 0,
        // not vanish from the list.
        assert_eq!(find(&views, postless_feed.id).unread_count, 0);

        Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
        Feed::delete(&pool, postless_feed.id)
            .await
            .expect("should delete feed");
    }
}
