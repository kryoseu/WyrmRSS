use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::{
    models::webhook::Webhook,
    newtypes::FeedId,
    views::{self, webhook::FeedWebhookView},
};
use wyrm_utils::result::WyrmResult;

pub async fn list(ctx: Data<WyrmContext>) -> WyrmResult<Json<Vec<Webhook>>> {
    let webhooks = Webhook::get_all(&ctx.db_pool).await?;
    Ok(Json(webhooks))
}

pub async fn list_for_feed(
    path: Path<FeedId>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Vec<FeedWebhookView>>> {
    let webhooks = views::webhook::list_for_feed(&ctx.db_pool, path.into_inner()).await?;
    Ok(Json(webhooks))
}
