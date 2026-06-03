use actix_web::web::{Data, Json, Path, Query};
use api_utils::context::WyrmContext;
use database::{
    models::post::Post,
    utils::pagination::PagedResponse,
    views::post::{ListPosts, PostQuery},
};
use wyrm_utils::result::WyrmResult;

pub async fn get(path: Path<i32>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Post>> {
    let post_id = path.into_inner();
    let post = Post::get(&ctx.db_pool, post_id).await?;
    Ok(Json(post))
}

pub async fn list(
    query: Query<ListPosts>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PagedResponse<Vec<Post>>>> {
    let query = query.into_inner();
    let page = PostQuery {
        cursor: query.page,
        tag: query.tag,
        search: query.search,
        ..Default::default()
    }
    .list(&ctx.db_pool, ctx.settings.feed.page_size)
    .await?;
    Ok(Json(page))
}

pub async fn list_by_feed(
    path: Path<i32>,
    query: Query<ListPosts>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PagedResponse<Vec<Post>>>> {
    let query = query.into_inner();
    let page = PostQuery {
        feed_id: Some(path.into_inner()),
        cursor: query.page,
        search: query.search,
        ..Default::default()
    }
    .list(&ctx.db_pool, ctx.settings.feed.page_size)
    .await?;
    Ok(Json(page))
}

pub async fn list_favorites(
    query: Query<ListPosts>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PagedResponse<Vec<Post>>>> {
    let query = query.into_inner();
    let page = PostQuery {
        fav_only: true,
        cursor: query.page,
        ..Default::default()
    }
    .list(&ctx.db_pool, ctx.settings.feed.page_size)
    .await?;
    Ok(Json(page))
}
