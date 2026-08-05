use crate::{
    DatabasePool,
    models::expired::{ExpiredPost, ExpiredPostInsertForm},
    newtypes::{FeedId, FolderId, PostId},
    schema::{
        feeds,
        posts::{self},
    },
};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use feed_rs::model::Entry;
use serde::{Deserialize, Serialize};
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(crate::Backend))]
#[derive(ts_rs::TS)]
#[ts(optional_fields, export)]
/// A single entry fetched from a feed.
pub struct Post {
    /// Primary key.
    pub id: PostId,
    /// The feed this post belongs to; the row is removed when that feed is
    /// deleted (`ON DELETE CASCADE`).
    pub feed_id: FeedId,
    /// Post title, or `None` if the feed entry has none.
    pub title: Option<String>,
    /// Link to the original post. Forms the `(feed_id, url)` uniqueness
    /// constraint used to dedupe on re-fetch.
    pub url: Option<String>,
    /// Comma-separated author list, each formatted as `name (email)` (or just
    /// `name`); `None` if the entry lists no authors.
    pub authors: Option<String>,
    /// When the entry was published; defaults to insertion time if the feed
    /// omits it.
    pub published_at: DateTime<Utc>,
    /// When the entry was last updated, if the feed provides it.
    pub updated_at: Option<DateTime<Utc>>,
    /// Short summary or excerpt (the feed's summary, falling back to a media
    /// description for media feeds).
    pub description: Option<String>,
    /// Full post body, if the feed includes it.
    pub content: Option<String>,
    /// Whether the user has bookmarked this post.
    /// UI shows these as "Read Later".
    pub bookmarked: bool,
    /// Whether the post has been marked read.
    pub is_read: bool,
    /// Whether the post has been archived.
    pub is_archived: bool,
    /// When the entry was created - insertion time
    pub created_at: DateTime<Utc>,
}

impl Post {
    pub async fn get(pool: &DatabasePool, post_id: PostId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        posts::table
            .find(post_id)
            .select(Post::as_select())
            .first(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn create(pool: &DatabasePool, form: PostInsertForm) -> WyrmResult<()> {
        let mut conn = pool.get().await?;
        diesel::insert_into(posts::table)
            .values(form)
            .on_conflict((posts::feed_id, posts::url))
            .do_nothing()
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    /// Inserts many posts in a single statement.
    ///
    /// Rows that conflict on `(feed_id, url)` are silently skipped via
    /// `ON CONFLICT DO NOTHING`, so re-fetching a feed never errors on posts
    /// that already exist and never creates duplicates. The returned count is
    /// the number of rows actually inserted (conflicts excluded).
    ///
    /// Forms whose `(feed_id, url)` is recorded in `expired_posts` are also
    /// dropped before the insert, so posts expired (deleted) by the expiry  
    /// can't come back (see [`ExpiredPost`]).
    ///
    /// Because all rows share one statement, error granularity differs from
    /// inserting row-by-row: a genuine row-level failure (a constraint the
    /// database cannot skip, e.g. a NOT NULL / CHECK / foreign-key violation)
    /// or a connection error aborts the whole batch — none of these posts are
    /// inserted. Duplicate URLs are *not* such a failure; they are handled by
    /// the conflict clause above.
    ///
    /// An empty `forms` is a no-op and returns `Ok(0)` without touching the
    /// pool. A batch that becomes empty after the expired-posts filter is
    /// also fine.
    pub async fn create_many(
        pool: &DatabasePool,
        mut forms: Vec<PostInsertForm>,
    ) -> WyrmResult<Vec<Post>> {
        if forms.is_empty() {
            return Ok(vec![]);
        }
        let mut conn = pool.get().await?;

        // Filter out anything recorded in the expired_posts table before inserting.
        let feed_ids = forms.iter().map(|f| f.feed_id).collect();
        let urls = forms.iter().filter_map(|f| f.url.as_deref()).collect();
        let expired = ExpiredPost::matching(&mut conn, feed_ids, urls).await?;
        forms.retain(|f| match &f.url {
            Some(url) => !expired.contains(&(f.feed_id, url.clone())),
            None => true,
        });

        #[cfg(feature = "postgres")]
        {
            diesel::insert_into(posts::table)
                .values(&forms)
                .on_conflict((posts::feed_id, posts::url))
                .do_nothing()
                .returning(Post::as_select())
                .get_results(&mut conn)
                .await
                .map_err(WyrmError::from)
        }
        // SQLite cannot take a multi-row VALUES list here: `PostInsertForm`'s
        // `Option` fields make the values defaultable, and SQLite rejects the
        // `DEFAULT` keyword inside a multi-row insert. One statement per row
        // keeps `None` meaning "use the column default" exactly as on postgres.
        // Wrapped in a transaction so a mid-batch failure inserts nothing.
        #[cfg(feature = "sqlite")]
        {
            use diesel_async::AsyncConnection;
            let conn = &mut *conn;
            conn.transaction(async |conn| {
                let mut inserted = Vec::with_capacity(forms.len());
                for form in &forms {
                    // `do_nothing` means a conflicting row yields no row back,
                    // so an absent result is expected rather than an error.
                    let row = diesel::insert_into(posts::table)
                        .values(form)
                        .on_conflict((posts::feed_id, posts::url))
                        .do_nothing()
                        .returning(Post::as_select())
                        .get_result(conn)
                        .await
                        .optional()
                        .map_err(WyrmError::from)?;
                    inserted.extend(row);
                }
                Ok::<Vec<Post>, WyrmError>(inserted)
            })
            .await
        }
    }

    pub async fn update(pool: &DatabasePool, form: PostUpdateForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::update(posts::table.find(form.id))
            .set(form)
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn toggle_is_read(pool: &DatabasePool, post_id: PostId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::update(posts::table.find(post_id))
            .set(posts::is_read.eq(diesel::dsl::not(posts::is_read)))
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn mark_many_as_read(
        pool: &DatabasePool,
        feed_id: Option<FeedId>,
        folder_id: Option<FolderId>,
    ) -> WyrmResult<usize> {
        match (feed_id, folder_id) {
            (Some(feed_id), _) => Self::mark_feed_as_read(pool, feed_id).await,
            (None, Some(folder_id)) => Self::mark_folder_as_read(pool, folder_id).await,
            (None, None) => Self::mark_all_as_read(pool).await,
        }
    }

    async fn mark_all_as_read(pool: &DatabasePool) -> WyrmResult<usize> {
        let mut conn = pool.get().await?;
        diesel::update(posts::table)
            .filter(posts::is_read.eq(false))
            .set(posts::is_read.eq(true))
            .execute(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    async fn mark_feed_as_read(pool: &DatabasePool, feed_id: FeedId) -> WyrmResult<usize> {
        let mut conn = pool.get().await?;
        diesel::update(posts::table)
            .filter(posts::feed_id.eq(feed_id))
            .filter(posts::is_read.eq(false))
            .set(posts::is_read.eq(true))
            .execute(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    async fn mark_folder_as_read(pool: &DatabasePool, folder_id: FolderId) -> WyrmResult<usize> {
        let mut conn = pool.get().await?;
        let folder_feeds = feeds::table
            .filter(feeds::folder_id.eq(folder_id))
            .select(feeds::id);
        diesel::update(posts::table)
            .filter(posts::feed_id.eq_any(folder_feeds))
            .filter(posts::is_read.eq(false))
            .set(posts::is_read.eq(true))
            .execute(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn unread_count(pool: &DatabasePool, feed_id: FeedId) -> WyrmResult<i64> {
        let mut conn = pool.get().await?;
        posts::table
            .filter(posts::feed_id.eq(feed_id))
            .filter(posts::is_read.eq(false))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn expire_read(pool: &DatabasePool, days: i32) -> WyrmResult<usize> {
        Self::expire(pool, days, true).await
    }

    pub async fn expire_unread(pool: &DatabasePool, days: i32) -> WyrmResult<usize> {
        Self::expire(pool, days, false).await
    }
    /// Deletes read or unread posts older than `days` and tombstones their urls
    /// in `expired_posts` in the same transaction, so a later poll can't
    /// re-insert them while they're still listed in the upstream feed.
    async fn expire(pool: &DatabasePool, days: i32, is_read: bool) -> WyrmResult<usize> {
        let mut conn = pool.get().await?;
        let conn = &mut *conn;

        let cutoff = Utc::now() - Duration::days(days as i64);

        conn.transaction(async |conn| {
            let deleted: Vec<(FeedId, Option<String>)> = diesel::delete(
                posts::table
                    .filter(posts::is_read.eq(is_read))
                    .filter(posts::is_archived.eq(false))
                    .filter(posts::bookmarked.eq(false))
                    .filter(posts::created_at.lt(cutoff)),
            )
            .returning((posts::feed_id, posts::url))
            .get_results(conn)
            .await?;

            let count = deleted.len();
            let forms: Vec<ExpiredPostInsertForm> = deleted
                .into_iter()
                .filter_map(|(feed_id, url)| url.map(|url| ExpiredPostInsertForm { feed_id, url }))
                .collect();
            ExpiredPost::create_many_on(conn, forms).await?;

            Ok::<usize, WyrmError>(count)
        })
        .await
    }
}

#[derive(Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(crate::Backend))]
pub struct PostUpdateForm {
    pub id: PostId,
    pub bookmarked: Option<bool>,
    pub is_read: Option<bool>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(crate::Backend))]
pub struct PostInsertForm {
    pub feed_id: FeedId,
    pub title: Option<String>,
    pub url: Option<String>,
    pub authors: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl PostInsertForm {
    pub fn from_entry(entry: Entry, feed_id: FeedId) -> Self {
        let media_description = entry
            .media
            .into_iter()
            .next()
            .and_then(|m| m.description)
            .map(|d| d.content);
        Self {
            feed_id,
            title: entry.title.map(|t| t.content),
            url: entry.links.into_iter().next().map(|l| l.href),
            authors: if entry.authors.is_empty() {
                None
            } else {
                Some(
                    entry
                        .authors
                        .iter()
                        .map(|a| match &a.email {
                            Some(email) => format!("{} ({})", a.name, email),
                            None => a.name.clone(),
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            },
            published_at: entry.published,
            updated_at: entry.updated,
            description: entry.summary.map(|s| s.content).or(media_description),
            content: entry.content.and_then(|c| c.body),
            created_at: None,
        }
    }
}

/// Test helper: inserts a post under `feed_id` and returns the created `Post`.
/// Lives with the `Post` model and is shared with the archive tests via
/// `#[macro_use]` on this module in `models/mod.rs`.
#[cfg(test)]
macro_rules! test_post {
    ($pool:expr, $feed_id:expr) => {{
        let url = format!(
            "https://example.com/post/{}",
            chrono::Utc::now()
                .timestamp_nanos_opt()
                .expect("timestamp in range")
        );
        $crate::models::post::Post::create(
            $pool,
            $crate::models::post::PostInsertForm {
                feed_id: $feed_id,
                title: Some("test post".to_string()),
                url: Some(url.clone()),
                authors: None,
                published_at: Some(chrono::Utc::now()),
                updated_at: None,
                description: None,
                content: None,
                created_at: None,
            },
        )
        .await
        .expect("should create test post");

        let id = {
            use crate::newtypes::PostId;
            use diesel::prelude::*;
            use diesel_async::RunQueryDsl;
            let mut conn = $pool.get().await.expect("should get conn");
            $crate::schema::posts::table
                .filter($crate::schema::posts::feed_id.eq($feed_id))
                .filter($crate::schema::posts::url.eq(&url))
                .select($crate::schema::posts::id)
                .first::<PostId>(&mut conn)
                .await
                .expect("post should exist")
        };
        $crate::models::post::Post::get($pool, id)
            .await
            .expect("should get post")
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::feed::Feed, setup_test_db};
    use feed_rs::parser;

    macro_rules! unique {
        () => {
            Utc::now()
                .timestamp_nanos_opt()
                .expect("timestamp in range")
        };
    }

    /// A minimal insertable post with the given url.
    macro_rules! post_form {
        ($feed_id:expr, $url:expr) => {
            PostInsertForm {
                feed_id: $feed_id,
                title: Some("test post".to_string()),
                url: Some($url.to_string()),
                authors: None,
                published_at: Some(Utc::now()),
                updated_at: None,
                description: None,
                content: None,
                created_at: None,
            }
        };
    }

    fn atom_entry(entry_body: &str) -> Entry {
        let xml = format!(
            concat!(
                r#"<?xml version="1.0" encoding="utf-8"?>"#,
                r#"<feed xmlns="http://www.w3.org/2005/Atom" xmlns:media="http://search.yahoo.com/mrss/">"#,
                "<title>Test Feed</title><id>urn:feed</id><updated>2026-01-01T00:00:00Z</updated>",
                "<entry>{}</entry>",
                "</feed>",
            ),
            entry_body,
        );
        parser::parse(xml.as_bytes())
            .expect("fixture should parse")
            .entries
            .into_iter()
            .next()
            .expect("fixture should contain an entry")
    }

    #[test]
    fn maps_entry_fields() {
        let entry = atom_entry(concat!(
            "<id>urn:1</id><title>Hello World</title>",
            r#"<link href="https://example.com/post/1"/>"#,
            "<published>2026-01-02T03:04:05Z</published>",
            "<author><name>Jane Doe</name><email>jane@example.com</email></author>",
            "<summary>A short summary.</summary>",
            "<content>Full body.</content>",
        ));

        let form = PostInsertForm::from_entry(entry, FeedId(42));

        assert_eq!(form.feed_id, FeedId(42));
        assert_eq!(form.title.as_deref(), Some("Hello World"));
        assert_eq!(form.url.as_deref(), Some("https://example.com/post/1"));
        assert_eq!(form.authors.as_deref(), Some("Jane Doe (jane@example.com)"));
        assert_eq!(form.description.as_deref(), Some("A short summary."));
        assert_eq!(form.content.as_deref(), Some("Full body."));
        assert_eq!(
            form.published_at,
            Some("2026-01-02T03:04:05Z".parse::<DateTime<Utc>>().unwrap()),
        );
    }

    #[tokio::test]
    async fn get_returns_created_post() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);
        let url = format!("https://example.com/post/{}", unique!());

        Post::create(
            &pool,
            PostInsertForm {
                feed_id: feed.id,
                title: Some("Get Me".to_string()),
                url: Some(url.clone()),
                authors: None,
                published_at: Some("2026-01-02T03:04:05Z".parse::<DateTime<Utc>>().unwrap()),
                updated_at: None,
                description: Some("A description.".to_string()),
                content: Some("Body.".to_string()),
                created_at: None,
            },
        )
        .await
        .expect("should create post");

        let id: PostId = {
            let mut conn = pool.get().await.expect("should get conn");
            posts::table
                .filter(posts::feed_id.eq(feed.id))
                .filter(posts::url.eq(&url))
                .select(posts::id)
                .first(&mut conn)
                .await
                .expect("post should exist")
        };
        let got = Post::get(&pool, id).await.expect("get should succeed");

        assert_eq!(got.id, id);
        assert_eq!(got.feed_id, feed.id);
        assert_eq!(got.title.as_deref(), Some("Get Me"));
        assert_eq!(got.url.as_deref(), Some(url.as_str()));
        assert_eq!(got.description.as_deref(), Some("A description."));
        assert_eq!(got.content.as_deref(), Some("Body."));
        assert!(!got.is_read);
        assert!(!got.bookmarked);
        assert!(!got.is_archived);

        // Deleting the feed cascades to the post, so a follow-up get fails.
        Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
        assert!(Post::get(&pool, id).await.is_err());
    }

    #[tokio::test]
    async fn create_inserts_and_skips_conflicts() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);
        let url = format!("https://example.com/post/{}", unique!());

        Post::create(&pool, post_form!(feed.id, &url))
            .await
            .expect("first insert should succeed");
        // Same (feed_id, url) hits ON CONFLICT DO NOTHING: no error, no dup.
        Post::create(&pool, post_form!(feed.id, &url))
            .await
            .expect("conflicting insert should be a no-op");

        let count: i64 = {
            let mut conn = pool.get().await.expect("should get conn");
            posts::table
                .filter(posts::feed_id.eq(feed.id))
                .count()
                .get_result(&mut conn)
                .await
                .expect("should count posts")
        };
        assert_eq!(count, 1);

        Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
    }

    #[tokio::test]
    async fn create_many_inserts_batch_and_skips_conflicts() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);

        // Empty input is a no-op.
        assert_eq!(Post::create_many(&pool, vec![]).await.unwrap().len(), 0);

        let urls: Vec<String> = (0..3)
            .map(|i| format!("https://example.com/post/{}-{i}", unique!()))
            .collect();
        let forms: Vec<PostInsertForm> = urls.iter().map(|u| post_form!(feed.id, u)).collect();
        assert_eq!(Post::create_many(&pool, forms).await.unwrap().len(), 3);

        // Re-inserting the same urls all conflict, so nothing new is inserted.
        let dups: Vec<PostInsertForm> = urls.iter().map(|u| post_form!(feed.id, u)).collect();
        assert_eq!(Post::create_many(&pool, dups).await.unwrap().len(), 0);

        let count: i64 = {
            let mut conn = pool.get().await.expect("should get conn");
            posts::table
                .filter(posts::feed_id.eq(feed.id))
                .count()
                .get_result(&mut conn)
                .await
                .expect("should count posts")
        };
        assert_eq!(count, 3);

        Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
    }

    #[tokio::test]
    async fn update_changes_flags() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);
        let id = test_post!(&pool, feed.id).id;

        let updated = Post::update(
            &pool,
            PostUpdateForm {
                id,
                bookmarked: Some(true),
                is_read: Some(true),
            },
        )
        .await
        .expect("update should succeed");

        assert!(updated.bookmarked);
        assert!(updated.is_read);

        Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
    }

    #[tokio::test]
    async fn unread_count_ignores_read_posts_and_other_feeds() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);
        let other_feed = test_feed!(&pool);

        let _ = test_post!(&pool, feed.id);
        let _ = test_post!(&pool, feed.id);
        let read_id = test_post!(&pool, feed.id).id;
        test_post!(&pool, other_feed.id);

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

        assert_eq!(Post::unread_count(&pool, feed.id).await.unwrap(), 2);
        assert_eq!(Post::unread_count(&pool, other_feed.id).await.unwrap(), 1);

        // Deleting cascades the posts away; a postless feed counts 0.
        Feed::delete(&pool, other_feed.id)
            .await
            .expect("should delete feed");
        assert_eq!(Post::unread_count(&pool, other_feed.id).await.unwrap(), 0);

        Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
    }

    #[tokio::test]
    async fn toggle_is_read_flips_value() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);
        let id = test_post!(&pool, feed.id).id;

        // Defaults to false: first toggle -> true, second -> false.
        assert!(Post::toggle_is_read(&pool, id).await.unwrap().is_read);
        assert!(!Post::toggle_is_read(&pool, id).await.unwrap().is_read);

        Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
    }

    #[test]
    fn media_feed_entry_uses_media_description() {
        let entry = atom_entry(concat!(
            "<id>urn:video:1</id>",
            "<title>Test Video</title>",
            r#"<link rel="alternate" href="https://example.com/video/1"/>"#,
            "<author><name>Test Author</name><uri>https://example.com/author</uri></author>",
            "<published>2026-01-02T03:04:05Z</published>",
            "<media:group>",
            "<media:title>Test Video</media:title>",
            r#"<media:content url="https://example.com/video/1.mp4" type="video/mp4"/>"#,
            r#"<media:thumbnail url="https://example.com/video/1.jpg"/>"#,
            "<media:description>Test description.</media:description>",
            "</media:group>",
        ));

        let form = PostInsertForm::from_entry(entry, FeedId(7));

        assert_eq!(form.title.as_deref(), Some("Test Video"));
        assert_eq!(form.url.as_deref(), Some("https://example.com/video/1"));
        assert_eq!(form.authors.as_deref(), Some("Test Author"));
        assert_eq!(form.description.as_deref(), Some("Test description."));
        assert_eq!(form.content, None);
    }

    /// One test covers both sweeps and the refetch: the sweeps scan the whole
    /// (shared) test database, so parallel tests with backdated posts would
    /// expire each other's fixtures.
    #[tokio::test]
    async fn expire_records_expired_posts_and_blocks_refetch() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);

        // Look up the id a url got on insert.
        macro_rules! post_id {
            ($url:expr) => {{
                let mut conn = pool.get().await.expect("should get conn");
                posts::table
                    .filter(posts::feed_id.eq(feed.id))
                    .filter(posts::url.eq($url))
                    .select(posts::id)
                    .first::<PostId>(&mut conn)
                    .await
                    .expect("post should exist")
            }};
        }

        // Three posts old enough to expire: one read, one unread, and one
        // read + bookmarked that must survive the sweep.
        let read_url = format!("https://example.com/post/{}", unique!());
        let unread_url = format!("https://example.com/post/{}", unique!());
        let kept_url = format!("https://example.com/post/{}", unique!());
        for url in [&read_url, &unread_url, &kept_url] {
            let mut form = post_form!(feed.id, url);
            form.created_at = Some(Utc::now() - Duration::days(10));
            Post::create(&pool, form).await.expect("should create post");
        }
        let read_id = post_id!(&read_url);
        let kept_id = post_id!(&kept_url);
        for (id, bookmarked) in [(read_id, None), (kept_id, Some(true))] {
            Post::update(
                &pool,
                PostUpdateForm {
                    id,
                    bookmarked,
                    is_read: Some(true),
                },
            )
            .await
            .expect("should mark post read");
        }

        // Other tests share the database, so rows besides ours may expire:
        // assert on our posts, not exact counts.
        assert!(Post::expire_read(&pool, 5).await.unwrap() >= 1);
        assert!(Post::expire_unread(&pool, 5).await.unwrap() >= 1);
        assert!(Post::get(&pool, read_id).await.is_err());
        assert!(Post::get(&pool, kept_id).await.is_ok());
        assert_eq!(Post::unread_count(&pool, feed.id).await.unwrap(), 0);

        // Re-fetch: the expired urls must stay gone — their rows are deleted,
        // so only the expired_posts filter can block them — while a fresh url
        // in the same batch inserts normally.
        let fresh_url = format!("https://example.com/post/{}", unique!());
        let inserted = Post::create_many(
            &pool,
            vec![
                post_form!(feed.id, &read_url),
                post_form!(feed.id, &unread_url),
                post_form!(feed.id, &fresh_url),
            ],
        )
        .await
        .expect("create_many should succeed");
        assert_eq!(inserted.len(), 1);
        assert_eq!(inserted[0].url.as_deref(), Some(fresh_url.as_str()));

        Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
    }
}
