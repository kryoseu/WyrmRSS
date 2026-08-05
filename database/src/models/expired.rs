use crate::{DatabaseConn, newtypes::FeedId, schema::expired_posts};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashSet;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

/// History of expired (deleted) posts to prevent re-fetching.
/// Without this table, an expired post still advertised by the upstream feed
/// would be re-inserted as new on every poll, so `Post::create_many` drops
/// any incoming form recorded here. Rows cascade away with their feed,
/// so re-adding a feed starts with a clean state.
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::expired_posts)]
#[diesel(check_for_backend(crate::Backend))]
pub struct ExpiredPost {
    pub feed_id: FeedId,
    pub url: String,
}

impl ExpiredPost {
    /// Records deleted posts on an existing connection so `Post::expire`'s
    /// delete + record transaction are atomic. Re-adding an already
    /// recorded pair is a silent no-op (`ON CONFLICT DO NOTHING`).
    pub async fn create_many_on(
        conn: &mut DatabaseConn,
        forms: Vec<ExpiredPostInsertForm>,
    ) -> WyrmResult<usize> {
        #[cfg(feature = "postgres")]
        {
            diesel::insert_into(expired_posts::table)
                .values(&forms)
                .on_conflict_do_nothing()
                .execute(&mut *conn)
                .await
                .map_err(WyrmError::from)
        }
        // diesel's SQLite backend has no `BatchInsert` support for a values list
        // combined with ON CONFLICT, so insert row by row. Callers already run
        // this inside a transaction, which is what keeps the batch atomic.
        #[cfg(feature = "sqlite")]
        {
            let mut inserted = 0;
            for form in &forms {
                inserted += diesel::insert_into(expired_posts::table)
                    .values(form)
                    .on_conflict_do_nothing()
                    .execute(&mut *conn)
                    .await
                    .map_err(WyrmError::from)?;
            }
            Ok(inserted)
        }
    }

    /// Which of the candidate `(feed_id, url)` pairs are recorded as expired.
    /// Runs on an existing connection so `Post::create_many` doesn't need a
    /// second pool checkout.
    pub async fn matching(
        conn: &mut DatabaseConn,
        feed_ids: Vec<FeedId>,
        urls: Vec<&str>,
    ) -> WyrmResult<HashSet<(FeedId, String)>> {
        expired_posts::table
            .filter(expired_posts::feed_id.eq_any(feed_ids))
            .filter(expired_posts::url.eq_any(urls))
            .select((expired_posts::feed_id, expired_posts::url))
            .load::<(FeedId, String)>(&mut *conn)
            .await
            .map(|rows| rows.into_iter().collect())
            .map_err(WyrmError::from)
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::expired_posts)]
#[diesel(check_for_backend(crate::Backend))]
pub struct ExpiredPostInsertForm {
    pub feed_id: FeedId,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::feed::Feed, setup_test_db};

    macro_rules! unique_url {
        () => {
            format!(
                "https://example.com/expired/{}",
                chrono::Utc::now()
                    .timestamp_nanos_opt()
                    .expect("timestamp in range")
            )
        };
    }

    #[tokio::test]
    async fn create_skips_conflicts_matching_is_exact_and_feed_delete_cascades() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);
        let other_feed = test_feed!(&pool);
        let url_a = unique_url!();
        let url_b = unique_url!();

        let mut conn = pool.get().await.expect("should get conn");
        let forms = vec![
            ExpiredPostInsertForm {
                feed_id: feed.id,
                url: url_a.clone(),
            },
            ExpiredPostInsertForm {
                feed_id: feed.id,
                url: url_b.clone(),
            },
        ];
        assert_eq!(ExpiredPost::create_many_on(&mut conn, forms).await.unwrap(), 2);

        // Same (feed_id, url) hits ON CONFLICT DO NOTHING: no error, no dup.
        let dup = vec![ExpiredPostInsertForm {
            feed_id: feed.id,
            url: url_a.clone(),
        }];
        assert_eq!(ExpiredPost::create_many_on(&mut conn, dup).await.unwrap(), 0);

        // Exact pairs only: other_feed and an unknown url appear in the
        // filters, but neither (other_feed, url_a) nor (feed, unknown) is a
        // row in the table.
        let hits = ExpiredPost::matching(
            &mut conn,
            vec![feed.id, other_feed.id],
            vec![url_a.as_str(), "https://example.com/never-expired"],
        )
        .await
        .unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits.contains(&(feed.id, url_a.clone())));

        // Deleting the feed cascades its expired-post records away.
        Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
        let hits =
            ExpiredPost::matching(&mut conn, vec![feed.id], vec![url_a.as_str(), url_b.as_str()])
                .await
                .unwrap();
        assert!(hits.is_empty());

        Feed::delete(&pool, other_feed.id)
            .await
            .expect("should delete feed");
    }
}
