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
pub struct Post {
    pub id: i32,
    pub feed_id: i32,
    pub title: Option<String>,
    pub url: Option<String>,
    pub authors: Option<String>,
    pub published_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub is_favorite: bool,
    pub is_read: bool,
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
    use feed_rs::parser;

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
