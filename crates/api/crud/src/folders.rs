use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::{
    models::folder::{Folder, FolderUpdateForm},
    newtypes::{FolderId, UserId},
};
use serde::Deserialize;
use wyrm_utils::result::WyrmResult;

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CreateFolder {
    name: String,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdateFolder {
    name: String,
}

pub async fn create(
    Json(data): Json<CreateFolder>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Folder>> {
    ctx.deny_in_demo()?;

    // TODO(#372): the requesting user, once the middleware provides it.
    let folder = Folder::resolve_or_create(&ctx.db_pool, UserId(1), &data.name).await?;
    Ok(Json(folder))
}

pub async fn update(
    path: Path<FolderId>,
    Json(data): Json<UpdateFolder>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Folder>> {
    ctx.deny_in_demo()?;

    // TODO(#372): the requesting user, once the middleware provides it.
    let user_id = UserId(1);
    let folder = Folder::update(
        &ctx.db_pool,
        FolderUpdateForm {
            id: path.into_inner(),
            user_id,
            name: data.name,
        },
    )
    .await?;
    Ok(Json(folder))
}

pub async fn delete(path: Path<FolderId>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Folder>> {
    ctx.deny_in_demo()?;

    // TODO(#372): the requesting user, once the middleware provides it.
    let user_id = UserId(1);
    let folder = Folder::delete(&ctx.db_pool, user_id, path.into_inner()).await?;
    Ok(Json(folder))
}
