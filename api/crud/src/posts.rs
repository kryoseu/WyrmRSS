use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::{
    models::post::{Post, PostUpdateForm},
    newtypes::PostId,
};
use serde::{Deserialize, Serialize};
use wyrm_utils::result::WyrmResult;

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdatePost {
    is_favorite: Option<bool>,
    is_read: Option<bool>,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdatePostResponse {
    post: Post,
    feed_unread_count: i64,
}

pub async fn update(
    path: Path<PostId>,
    Json(data): Json<UpdatePost>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<UpdatePostResponse>> {
    let post = Post::update(
        &ctx.db_pool,
        PostUpdateForm {
            id: path.into_inner(),
            is_favorite: data.is_favorite,
            is_read: data.is_read,
        },
    )
    .await?;

    let feed_unread_count = Post::unread_count(&ctx.db_pool, post.feed_id).await?;

    Ok(Json(UpdatePostResponse {
        post,
        feed_unread_count,
    }))
}
