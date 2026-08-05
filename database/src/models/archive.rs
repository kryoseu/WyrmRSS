use crate::{
    DatabasePool,
    models::post::Post,
    newtypes::PostId,
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
#[diesel(check_for_backend(crate::Backend))]
#[derive(ts_rs::TS)]
#[ts(optional_fields, export)]
/// A snapshot of an archived post, kept independently of the `posts` table so
/// it survives the original post (and its feed) being deleted.
pub struct PostArchive {
    /// The original post's id, reused as this row's primary key — there is no
    /// foreign key back to `posts`, so the archive outlives the post.
    pub id: PostId,
    /// Post title at the time it was archived.
    pub title: Option<String>,
    /// Link to the original post.
    pub url: Option<String>,
    /// Comma-separated author list captured from the post.
    pub authors: Option<String>,
    /// The post's original publish timestamp.
    pub published_at: DateTime<Utc>,
    /// Short summary or excerpt captured from the post.
    pub description: Option<String>,
    /// Full post body captured from the post.
    pub content: Option<String>,
    /// When the post was archived.
    pub archived_at: DateTime<Utc>,
}

impl PostArchive {
    pub async fn get(pool: &DatabasePool, id: PostId) -> WyrmResult<Self> {
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

            Ok::<PostArchive, WyrmError>(archived)
        })
        .await
    }

    /// Unarchives a post: deletes the `post_archive` row and sets `posts.is_archived = false`
    /// atomically. If the original post no longer exists (e.g. feed was deleted), the archive
    /// row is still removed.
    pub async fn delete(pool: &DatabasePool, post_id: PostId) -> WyrmResult<()> {
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
#[diesel(check_for_backend(crate::Backend))]
pub struct PostArchiveInsertForm {
    pub id: PostId,
    pub title: Option<String>,
    pub url: Option<String>,
    pub authors: Option<String>,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub content: Option<String>,
}

impl From<Post> for PostArchiveInsertForm {
    fn from(p: Post) -> Self {
        PostArchiveInsertForm {
            id: p.id,
            title: p.title,
            url: p.url,
            authors: p.authors,
            published_at: p.published_at,
            description: p.description,
            content: p.content,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::feed::Feed, setup_test_db};

    #[tokio::test]
    async fn archive_and_unarchive_roundtrip() {
        let pool = setup_test_db().await;
        let feed = test_feed!(&pool);
        let feed_id = feed.id;
        let post = test_post!(&pool, feed_id);
        let post_id = post.id;

        let archived = PostArchive::create(&pool, post.into())
            .await
            .expect("archive should succeed");
        assert_eq!(archived.id, post_id);
        assert_eq!(archived.title.as_deref(), Some("test post"));
        assert!(Post::get(&pool, post_id).await.unwrap().is_archived);
        assert_eq!(
            PostArchive::get(&pool, post_id).await.unwrap().id,
            post_id,
            "get should return the archived row"
        );

        PostArchive::delete(&pool, post_id)
            .await
            .expect("unarchive should succeed");
        assert!(PostArchive::get(&pool, post_id).await.is_err());
        assert!(!Post::get(&pool, post_id).await.unwrap().is_archived);

        Feed::delete(&pool, feed_id)
            .await
            .expect("should delete feed");
    }
}
