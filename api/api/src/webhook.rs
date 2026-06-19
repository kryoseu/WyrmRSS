use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::{models::webhook::Webhook, views};
use wyrm_utils::result::WyrmResult;

pub async fn get(path: Path<i32>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Webhook>> {
    let webhook = Webhook::get(&ctx.db_pool, path.into_inner()).await?;
    Ok(Json(webhook))
}

pub async fn list(ctx: Data<WyrmContext>) -> WyrmResult<Json<Vec<Webhook>>> {
    let webhooks = Webhook::get_all(&ctx.db_pool).await?;
    Ok(Json(webhooks))
}

pub async fn list_for_feed(
    path: Path<i32>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Vec<Webhook>>> {
    let webhooks = views::webhook::for_feed(&ctx.db_pool, path.into_inner()).await?;
    Ok(Json(webhooks))
}
