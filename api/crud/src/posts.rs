use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::{
    models::post::{Post, PostUpdateForm},
    newtypes::PostId,
};
use serde::Deserialize;
use wyrm_utils::result::WyrmResult;

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdatePost {
    is_favorite: Option<bool>,
    is_read: Option<bool>,
}

pub async fn update(
    path: Path<PostId>,
    Json(data): Json<UpdatePost>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Post>> {
    let post = Post::update(
        &ctx.db_pool,
        PostUpdateForm {
            id: path.into_inner(),
            is_favorite: data.is_favorite,
            is_read: data.is_read,
        },
    )
    .await?;
    Ok(Json(post))
}
