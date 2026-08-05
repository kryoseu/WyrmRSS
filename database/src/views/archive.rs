use crate::{
    DatabasePool,
    models::{
        archive::{PostArchive, PostArchiveInsertForm},
        post::Post,
    },
    newtypes::PostId,
    schema::post_archive,
    utils::pagination::{PagedResponse, PaginationCursor},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use wyrm_utils::{
    error::{DatabaseError, WyrmError},
    result::WyrmResult,
};

#[derive(Deserialize, ts_rs::TS)]
#[ts(optional_fields, export)]
pub struct ListPostArchive {
    pub page: Option<PaginationCursor>,
    pub search: Option<String>,
}

#[derive(Default)]
pub struct PostArchiveQuery {
    pub search: Option<String>,
    pub cursor: Option<PaginationCursor>,
}

impl PostArchiveQuery {
    pub async fn list(
        self,
        pool: &DatabasePool,
        page_size: i32,
    ) -> WyrmResult<PagedResponse<Vec<PostArchive>>> {
        let mut conn = pool.get().await?;

        let cursor = self.cursor.as_ref().and_then(|c| c.decode());

        let mut query = post_archive::table
            .select(PostArchive::as_select())
            .order((post_archive::archived_at.desc(), post_archive::id.desc()))
            .limit(page_size as i64 + 1)
            .into_boxed();

        if let Some(search) = self.search {
            query = query.filter(
                crate::ci_like!(post_archive::title, format!("%{search}%"))
                    .or(crate::ci_like!(post_archive::description, format!("%{search}%"))),
            );
        }

        if let Some((timestamp, post_id)) = cursor {
            query = query.filter(
                post_archive::archived_at
                    .lt(timestamp)
                    .or(post_archive::archived_at
                        .eq(timestamp)
                        .and(post_archive::id.lt(post_id))),
            );
        }

        let mut items: Vec<PostArchive> = query.load(&mut conn).await.map_err(WyrmError::from)?;
        let next_page = if items.len() > page_size as usize {
            items.truncate(page_size as usize);
            items
                .last()
                .map(|p| PaginationCursor::encode(p.archived_at, p.id.0))
        } else {
            None
        };

        Ok(PagedResponse { items, next_page })
    }
}

pub async fn get_post_archive_insert_form(
    pool: &DatabasePool,
    post_id: PostId,
) -> WyrmResult<PostArchiveInsertForm> {
    use crate::schema::posts;

    let mut conn = pool.get().await?;

    let post: Post = posts::table
        .find(post_id)
        .select(Post::as_select())
        .first(&mut conn)
        .await
        .map_err(WyrmError::from)?;

    if post.is_archived {
        return Err(WyrmError::Database(DatabaseError::Conflict(
            "post already archived".into(),
        )));
    }

    Ok(post.into())
}
