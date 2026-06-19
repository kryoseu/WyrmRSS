use actix_web::{
    HttpResponse,
    web::{Data, Json, Path},
};
use api_utils::context::WyrmContext;
use database::models::webhook::{
    FeedWebhook,
    Webhook,
    WebhookInsertForm,
    WebhookKind,
    WebhookUpdateForm,
};
use serde::Deserialize;
use wyrm_utils::{error::WyrmError, result::WyrmResult};
use wyrm_webhook as webhook;

#[derive(Deserialize, ts_rs::TS)]
#[ts(optional_fields, export)]
pub struct CreateWebhook {
    name: String,
    url: String,
    kind: WebhookKind,
    payload_template: Option<String>,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(optional_fields, export)]
pub struct UpdateWebhook {
    name: Option<String>,
    url: Option<String>,
    kind: WebhookKind,
    payload_template: Option<String>,
}

pub async fn create(
    Json(data): Json<CreateWebhook>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Webhook>> {
    webhook::validate(&data.kind, data.payload_template.as_deref())
        .map_err(|e| WyrmError::WebhookTemplate(e.to_string()))?;

    let webhook = Webhook::create(
        &ctx.db_pool,
        WebhookInsertForm {
            name: data.name,
            url: data.url,
            kind: data.kind,
            payload_template: data.payload_template,
        },
    )
    .await?;
    Ok(Json(webhook))
}

pub async fn update(
    path: Path<i32>,
    Json(data): Json<UpdateWebhook>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Webhook>> {
    // An unchanged template was already validated when the webhook was created,
    // so only re-validate when the caller supplies a new one.
    if let Some(template) = &data.payload_template {
        webhook::validate(&data.kind, Some(template))
            .map_err(|e| WyrmError::WebhookTemplate(e.to_string()))?;
    }

    let webhook = Webhook::update(
        &ctx.db_pool,
        WebhookUpdateForm {
            id: path.into_inner(),
            name: data.name,
            url: data.url,
            kind: Some(data.kind),
            payload_template: data.payload_template,
        },
    )
    .await?;
    Ok(Json(webhook))
}

pub async fn delete(path: Path<i32>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Webhook>> {
    let webhook = Webhook::delete(&ctx.db_pool, path.into_inner()).await?;
    Ok(Json(webhook))
}

pub async fn attach(path: Path<(i32, i32)>, ctx: Data<WyrmContext>) -> WyrmResult<HttpResponse> {
    let (feed_id, webhook_id) = path.into_inner();
    FeedWebhook::create(&ctx.db_pool, feed_id, webhook_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn detach(path: Path<(i32, i32)>, ctx: Data<WyrmContext>) -> WyrmResult<HttpResponse> {
    let (feed_id, webhook_id) = path.into_inner();
    FeedWebhook::delete(&ctx.db_pool, feed_id, webhook_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
