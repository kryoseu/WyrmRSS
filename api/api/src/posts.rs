use actix_web::web::{Data, Json, Path, Query};
use api_utils::context::WyrmContext;
use database::models::post::{Post, PostsPage};
use database::views::post::PostQuery;
use utils::result::WyrmResult;

use crate::CursorQuery;

pub async fn get(path: Path<i32>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Post>> {
    let post_id = path.into_inner();
    let post = Post::get(&ctx.db_pool, post_id).await?;
    Ok(Json(post))
}

pub async fn list(
    query: Query<CursorQuery>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PostsPage>> {
    let page = PostQuery {
        cursor: query.to_cursor(),
        ..Default::default()
    }
    .list(&ctx.db_pool, ctx.settings.feed.page_size)
    .await?;
    Ok(Json(page))
}

pub async fn list_by_feed(
    path: Path<i32>,
    query: Query<CursorQuery>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PostsPage>> {
    let feed_id = path.into_inner();
    let page = PostQuery {
        feed_id: Some(feed_id),
        cursor: query.to_cursor(),
        ..Default::default()
    }
    .list(&ctx.db_pool, ctx.settings.feed.page_size)
    .await?;
    Ok(Json(page))
}

pub async fn list_favorites(
    query: Query<CursorQuery>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PostsPage>> {
    let page = PostQuery {
        feed_id: None,
        fav_only: true,
        cursor: query.to_cursor(),
    }
    .list(&ctx.db_pool, ctx.settings.feed.page_size)
    .await?;
    Ok(Json(page))
}
