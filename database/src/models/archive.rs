use crate::{
    DatabasePool,
    models::{feed::Feed, post::Post},
    schema::{
        post_archive,
        posts::{self},
    },
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::Serialize;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::post_archive)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(ts_rs::TS)]
#[ts(optional_fields, export)]
pub struct PostArchive {
    pub id: i32,
    pub title: Option<String>,
    pub url: Option<String>,
    pub authors: Option<String>,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub tag: Option<String>,
    pub tag_color: Option<String>,
    pub archived_at: DateTime<Utc>,
}

impl PostArchive {
    pub async fn get(pool: &DatabasePool, id: i32) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        post_archive::table
            .find(id)
            .select(PostArchive::as_select())
            .first(&mut conn)
            .await
            .map_err(WyrmError::from)
    }
    /// Archives a post: inserts into `post_archive` and sets `posts.is_archived = true` atomically.
    /// Returns the created archive record including its generated `id` and `archived_at`.
    pub async fn create(
        pool: &DatabasePool,
        form: PostArchiveInsertForm,
    ) -> WyrmResult<PostArchive> {
        let mut conn = pool.get().await?;
        let conn = &mut *conn;

        let id = form.id;
        conn.transaction(async |conn| {
            let archived = diesel::insert_into(post_archive::table)
                .values(form)
                .get_result::<PostArchive>(conn)
                .await?;

            diesel::update(posts::table.find(id))
                .set(posts::is_archived.eq(true))
                .execute(conn)
                .await?;

            Ok::<PostArchive, diesel::result::Error>(archived)
        })
        .await
        .map_err(Into::into)
    }

    /// Unarchives a post: deletes the `post_archive` row and sets `posts.is_archived = false`
    /// atomically. If the original post no longer exists (e.g. feed was deleted), the archive
    /// row is still removed.
    pub async fn delete(pool: &DatabasePool, post_id: i32) -> WyrmResult<()> {
        let mut conn = pool.get().await?;
        let conn = &mut *conn;

        conn.transaction(async |conn| {
            diesel::delete(post_archive::table.filter(post_archive::id.eq(post_id)))
                .execute(conn)
                .await?;

            diesel::update(posts::table.find(post_id))
                .set(posts::is_archived.eq(false))
                .execute(conn)
                .await
                .ok();

            Ok::<(), diesel::result::Error>(())
        })
        .await
        .map_err(Into::into)
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::post_archive)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PostArchiveInsertForm {
    pub id: i32,
    pub title: Option<String>,
    pub url: Option<String>,
    pub authors: Option<String>,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub tag: Option<String>,
    pub tag_color: Option<String>,
}

impl From<(Post, Feed)> for PostArchiveInsertForm {
    fn from((p, f): (Post, Feed)) -> Self {
        PostArchiveInsertForm {
            id: p.id,
            title: p.title,
            url: p.url,
            authors: p.authors,
            published_at: p.published_at,
            description: p.description,
            content: p.content,
            tag: f.tag,
            tag_color: f.tag_color,
        }
    }
}
