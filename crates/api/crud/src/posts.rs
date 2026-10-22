use actix_web::web::{Data, Json, Path};
use api_utils::{context::WyrmContext, response::UpdatePostResponse};
use database::{
    models::post::{Post, PostUpdateForm},
    newtypes::PostId,
};
use serde::Deserialize;
use wyrm_utils::result::WyrmResult;

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdatePost {
    bookmarked: Option<bool>,
    is_read: Option<bool>,
}

pub async fn update(
    path: Path<PostId>,
    Json(data): Json<UpdatePost>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<UpdatePostResponse>> {
    let (post, feed_unread_count) = Post::update_with_unread_count(
        &ctx.db_pool,
        PostUpdateForm {
            id: path.into_inner(),
            bookmarked: data.bookmarked,
            is_read: data.is_read,
        },
    )
    .await?;

    Ok(Json(UpdatePostResponse {
        post,
        feed_unread_count,
    }))
}
