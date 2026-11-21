use actix_web::web::{Data, Json, Path, Query};
use api_utils::context::WyrmContext;
use database::{
    models::archive::PostArchive,
    newtypes::{PostId, UserId},
    utils::pagination::PagedResponse,
    views::archive::{ListPostArchive, PostArchiveQuery},
};
use wyrm_utils::result::WyrmResult;

pub async fn get(path: Path<PostId>, ctx: Data<WyrmContext>) -> WyrmResult<Json<PostArchive>> {
    // TODO(#372): the requesting user, once the middleware provides it.
    let user_id = UserId(1);
    let post_id = path.into_inner();
    let archive = PostArchive::get(&ctx.db_pool, user_id, post_id).await?;
    Ok(Json(archive))
}

pub async fn list(
    query: Query<ListPostArchive>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PagedResponse<Vec<PostArchive>>>> {
    // TODO(#372): the requesting user, once the middleware provides it.
    let user_id = UserId(1);
    let query = query.into_inner();
    let page_size = ctx.runtime_settings.read()?.page_size;
    let page = PostArchiveQuery {
        cursor: query.page,
        search: query.search,
    }
    .list(&ctx.db_pool, user_id, page_size)
    .await?;
    Ok(Json(page))
}
