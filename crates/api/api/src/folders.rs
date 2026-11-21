use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::{
    models::folder::Folder,
    newtypes::{FolderId, UserId},
};
use wyrm_utils::result::WyrmResult;

pub async fn get(path: Path<FolderId>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Folder>> {
    // TODO(#372): the requesting user, once the middleware provides it.
    let user_id = UserId(1);
    let folder = Folder::get(&ctx.db_pool, user_id, path.into_inner()).await?;
    Ok(Json(folder))
}

pub async fn list(ctx: Data<WyrmContext>) -> WyrmResult<Json<Vec<Folder>>> {
    // TODO(#372): the requesting user, once the middleware provides it.
    let user_id = UserId(1);
    let folders = Folder::get_all(&ctx.db_pool, user_id).await?;
    Ok(Json(folders))
}
