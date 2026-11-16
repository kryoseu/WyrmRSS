use crate::{
    DatabasePool,
    models::feed::Feed,
    newtypes::FeedId,
    schema::{feed_icons, feeds, posts},
};
use diesel::{dsl::count, prelude::*};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use std::collections::HashSet;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FeedView {
    #[serde(flatten)]
    pub feed: Feed,
    pub unread_count: i64,
    /// Whether `GET /feeds/{id}/icon` will serve an image.
    pub has_icon: bool,
}

/// List of feeds with post unread count
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

    let icon_ids: HashSet<FeedId> = feed_icons::table
        .filter(feed_icons::data.is_not_null())
        .select(feed_icons::feed_id)
        .load::<FeedId>(&mut conn)
        .await
        .map_err(WyrmError::from)?
        .into_iter()
        .collect();

    Ok(rows
        .into_iter()
        .map(|(feed, unread_count)| FeedView {
            has_icon: icon_ids.contains(&feed.id),
            feed,
            unread_count,
        })
        .collect())
}

#[cfg(all(test, feature = "postgres"))]
mod tests {
    use super::*;
    use crate::{
        models::{
            feed_icon::{FeedIcon, FeedIconInsertForm},
            post::{Post, PostUpdateForm},
        },
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
                bookmarked: None,
                is_read: Some(true),
            },
        )
        .await
        .expect("should mark post read");

        // Only stored bytes flag has_icon; a "checked, nothing found" row
        // must not.
        FeedIcon::create(
            &pool,
            FeedIconInsertForm {
                feed_id: feed.id,
                data: Some(vec![1]),
                content_type: Some("image/png".into()),
            },
        )
        .await
        .expect("should store icon");
        FeedIcon::create(
            &pool,
            FeedIconInsertForm {
                feed_id: postless_feed.id,
                data: None,
                content_type: None,
            },
        )
        .await
        .expect("should store icon lookup");

        let views = list(&pool).await.expect("list should succeed");
        assert_eq!(find(&views, feed.id).unread_count, 2);
        // A feed with no unread rows must survive the left join with a 0,
        // not vanish from the list.
        assert_eq!(find(&views, postless_feed.id).unread_count, 0);
        assert!(find(&views, feed.id).has_icon);
        assert!(!find(&views, postless_feed.id).has_icon);

        Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
        Feed::delete(&pool, postless_feed.id)
            .await
            .expect("should delete feed");
    }
}
