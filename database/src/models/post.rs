use crate::{
    DatabasePool,
    schema::posts::{self},
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use feed_rs::model::Entry;
use serde::Serialize;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(ts_rs::TS)]
#[ts(optional_fields, export)]
/// A single entry fetched from a feed.
pub struct Post {
    /// Primary key.
    pub id: i32,
    /// The feed this post belongs to; the row is removed when that feed is
    /// deleted (`ON DELETE CASCADE`).
    pub feed_id: i32,
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
    /// Whether the user has favorited this post.
    pub is_favorite: bool,
    /// Whether the post has been marked read.
    pub is_read: bool,
    /// Whether the post has been archived.
    pub is_archived: bool,
}

impl Post {
    pub async fn get(pool: &DatabasePool, post_id: i32) -> WyrmResult<Self> {
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
    /// Because all rows share one statement, error granularity differs from
    /// inserting row-by-row: a genuine row-level failure (a constraint the
    /// database cannot skip, e.g. a NOT NULL / CHECK / foreign-key violation)
    /// or a connection error aborts the whole batch — none of these posts are
    /// inserted. Duplicate URLs are *not* such a failure; they are handled by
    /// the conflict clause above.
    ///
    /// An empty `forms` is a no-op and returns `Ok(0)` without touching the
    /// pool.
    pub async fn create_many(pool: &DatabasePool, forms: Vec<PostInsertForm>) -> WyrmResult<usize> {
        if forms.is_empty() {
            return Ok(0);
        }
        let mut conn = pool.get().await?;
        let inserted = diesel::insert_into(posts::table)
            .values(&forms)
            .on_conflict((posts::feed_id, posts::url))
            .do_nothing()
            .execute(&mut conn)
            .await?;
        Ok(inserted)
    }

    pub async fn update(pool: &DatabasePool, form: PostUpdateForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::update(posts::table.find(form.id))
            .set(form)
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn toggle_is_read(pool: &DatabasePool, post_id: i32) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::update(posts::table.find(post_id))
            .set(posts::is_read.eq(diesel::dsl::not(posts::is_read)))
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }
}

#[derive(Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PostUpdateForm {
    pub id: i32,
    pub is_favorite: Option<bool>,
    pub is_read: Option<bool>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PostInsertForm {
    pub feed_id: i32,
    pub title: Option<String>,
    pub url: Option<String>,
    pub authors: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub content: Option<String>,
}

impl PostInsertForm {
    pub fn from_entry(entry: Entry, feed_id: i32) -> Self {
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
        }
    }
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

        let form = PostInsertForm::from_entry(entry, 42);

        assert_eq!(form.feed_id, 42);
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
            },
        )
        .await
        .expect("should create post");

        let id: i32 = {
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
        assert!(!got.is_favorite);
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
        assert_eq!(Post::create_many(&pool, vec![]).await.unwrap(), 0);

        let urls: Vec<String> = (0..3)
            .map(|i| format!("https://example.com/post/{}-{i}", unique!()))
            .collect();
        let forms: Vec<PostInsertForm> = urls.iter().map(|u| post_form!(feed.id, u)).collect();
        assert_eq!(Post::create_many(&pool, forms).await.unwrap(), 3);

        // Re-inserting the same urls all conflict, so nothing new is inserted.
        let dups: Vec<PostInsertForm> = urls.iter().map(|u| post_form!(feed.id, u)).collect();
        assert_eq!(Post::create_many(&pool, dups).await.unwrap(), 0);

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
        let url = format!("https://example.com/post/{}", unique!());
        Post::create(&pool, post_form!(feed.id, &url))
            .await
            .expect("should create test post");
        let id: i32 = {
            let mut conn = pool.get().await.expect("should get conn");
            posts::table
                .filter(posts::feed_id.eq(feed.id))
                .filter(posts::url.eq(&url))
                .select(posts::id)
                .first(&mut conn)
                .await
                .expect("post should exist")
        };

        let updated = Post::update(
            &pool,
            PostUpdateForm {
                id,
                is_favorite: Some(true),
                is_read: Some(true),
            },
        )
        .await
        .expect("update should succeed");

        assert!(updated.is_favorite);
        assert!(updated.is_read);

        Feed::delete(&pool, feed.id)
            .await
            .expect("should delete feed");
    }

    #[tokio::test]
    async fn toggle_is_read_flips_value() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);
        let url = format!("https://example.com/post/{}", unique!());
        Post::create(&pool, post_form!(feed.id, &url))
            .await
            .expect("should create test post");
        let id: i32 = {
            let mut conn = pool.get().await.expect("should get conn");
            posts::table
                .filter(posts::feed_id.eq(feed.id))
                .filter(posts::url.eq(&url))
                .select(posts::id)
                .first(&mut conn)
                .await
                .expect("post should exist")
        };

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

        let form = PostInsertForm::from_entry(entry, 7);

        assert_eq!(form.title.as_deref(), Some("Test Video"));
        assert_eq!(form.url.as_deref(), Some("https://example.com/video/1"));
        assert_eq!(form.authors.as_deref(), Some("Test Author"));
        assert_eq!(form.description.as_deref(), Some("Test description."));
        assert_eq!(form.content, None);
    }
}
