use actix_web::{
    HttpResponse,
    web::{Data, Json, Path, Query},
};
use api_utils::context::WyrmContext;
use database::{
    models::{archive::PostArchive, post::Post},
    utils::pagination::PagedResponse,
    views::{
        archive::get_post_archive_insert_form,
        post::{ListPosts, PostQuery},
    },
};
use wyrm_utils::result::WyrmResult;

pub async fn get(path: Path<i32>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Post>> {
    let post_id = path.into_inner();
    let post = Post::get(&ctx.db_pool, post_id).await?;
    Ok(Json(post))
}

pub async fn archive(path: Path<i32>, ctx: Data<WyrmContext>) -> WyrmResult<Json<PostArchive>> {
    let post_id = path.into_inner();
    let form = get_post_archive_insert_form(&ctx.db_pool, post_id).await?;
    let archived = PostArchive::create(&ctx.db_pool, form).await?;
    Ok(Json(archived))
}

pub async fn unarchive(path: Path<i32>, ctx: Data<WyrmContext>) -> WyrmResult<HttpResponse> {
    let post_id = path.into_inner();
    PostArchive::delete(&ctx.db_pool, post_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn list(
    query: Query<ListPosts>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PagedResponse<Vec<Post>>>> {
    let query = query.into_inner();
    let page_size = ctx.runtime_settings.read()?.page_size;
    let page = PostQuery {
        cursor: query.page,
        tag: query.tag,
        search: query.search,
        ..Default::default()
    }
    .list(&ctx.db_pool, page_size)
    .await?;
    Ok(Json(page))
}

pub async fn list_by_feed(
    path: Path<i32>,
    query: Query<ListPosts>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PagedResponse<Vec<Post>>>> {
    let query = query.into_inner();
    let page_size = ctx.runtime_settings.read()?.page_size;
    let page = PostQuery {
        feed_id: Some(path.into_inner()),
        cursor: query.page,
        search: query.search,
        ..Default::default()
    }
    .list(&ctx.db_pool, page_size)
    .await?;
    Ok(Json(page))
}

pub async fn list_favorites(
    query: Query<ListPosts>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PagedResponse<Vec<Post>>>> {
    let query = query.into_inner();
    let page_size = ctx.runtime_settings.read()?.page_size;
    let page = PostQuery {
        fav_only: true,
        cursor: query.page,
        ..Default::default()
    }
    .list(&ctx.db_pool, page_size)
    .await?;
    Ok(Json(page))
}
