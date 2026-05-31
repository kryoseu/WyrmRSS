use actix_web::web::{Data, Json, Path, Query};
use api_utils::context::WyrmContext;
use database::models::post::{Post, PostsPage};
use database::views::post::PostView;
use serde::Deserialize;
use utils::result::WyrmResult;

use crate::CursorQuery;

#[derive(Deserialize)]
pub struct PostListQuery {
    #[serde(flatten)]
    cursor: CursorQuery,
    tag: Option<String>,
}

pub async fn get(path: Path<i32>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Post>> {
    let post_id = path.into_inner();
    let post = Post::get(&ctx.db_pool, post_id).await?;
    Ok(Json(post))
}

pub async fn list(
    query: Query<PostListQuery>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PostsPage>> {
    let page = PostView {
        cursor: query.cursor.to_cursor(),
        tag: query.tag.clone(),
        ..Default::default()
    }
    .list(&ctx.db_pool, ctx.settings.feed.page_size)
    .await?;
    Ok(Json(page))
}

pub async fn list_by_feed(
    path: Path<i32>,
    query: Query<PostListQuery>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PostsPage>> {
    let feed_id = path.into_inner();
    let page = PostView {
        feed_id: Some(feed_id),
        cursor: query.cursor.to_cursor(),
        ..Default::default()
    }
    .list(&ctx.db_pool, ctx.settings.feed.page_size)
    .await?;
    Ok(Json(page))
}

pub async fn list_favorites(
    query: Query<PostListQuery>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PostsPage>> {
    let page = PostView {
        fav_only: true,
        cursor: query.cursor.to_cursor(),
        ..Default::default()
    }
    .list(&ctx.db_pool, ctx.settings.feed.page_size)
    .await?;
    Ok(Json(page))
}
