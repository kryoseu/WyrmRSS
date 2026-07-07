use crate::{
    DatabasePool,
    models::post::Post,
    newtypes::FeedId,
    schema::{feeds, posts},
    utils::pagination::{PagedResponse, PaginationCursor},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[derive(Default)]
pub struct PostQuery {
    pub feed_id: Option<FeedId>,
    pub tag: Option<String>,
    pub bookmarked: Option<bool>,
    pub unread_only: Option<bool>,
    pub search: Option<String>,
    pub exclude: Option<Vec<FeedId>>,
    pub cursor: Option<PaginationCursor>,
}

impl PostQuery {
    pub async fn list(
        self,
        pool: &DatabasePool,
        page_size: i32,
    ) -> WyrmResult<PagedResponse<Vec<Post>>> {
        let mut conn = pool.get().await?;

        let cursor = self.cursor.as_ref().and_then(|c| c.decode());

        let mut query = posts::table
            .select(Post::as_select())
            .order((posts::created_at.desc(), posts::id.desc()))
            .limit(page_size as i64 + 1)
            .into_boxed();

        if let Some(feed_id) = self.feed_id {
            query = query.filter(posts::feed_id.eq(feed_id));
        }

        if let Some(tag) = self.tag {
            query = query.filter(
                posts::feed_id.eq_any(feeds::table.select(feeds::id).filter(feeds::tag.eq(tag))),
            );
        }

        if let Some(search) = self.search {
            query = query.filter(
                posts::title
                    .ilike(format!("%{search}%"))
                    .or(posts::description.ilike(format!("%{search}%"))),
            );
        }

        if let Some(exclude) = self.exclude {
            query = query.filter(posts::feed_id.ne_all(exclude))
        }

        if self.bookmarked == Some(true) {
            query = query.filter(posts::bookmarked.eq(true));
        }

        if self.unread_only == Some(true) {
            query = query.filter(posts::is_read.eq(false));
        }

        if let Some((timestamp, post_id)) = cursor {
            query = query.filter(
                posts::created_at
                    .lt(timestamp)
                    .or(posts::created_at.eq(timestamp).and(posts::id.lt(post_id))),
            );
        }

        let mut items: Vec<Post> = query.load(&mut conn).await.map_err(WyrmError::from)?;
        let next_page = if items.len() > page_size as usize {
            items.truncate(page_size as usize);
            items
                .last()
                .map(|p| PaginationCursor::encode(p.created_at, p.id.0))
        } else {
            None
        };

        Ok(PagedResponse { items, next_page })
    }
}
