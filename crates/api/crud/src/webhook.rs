use actix_web::{
    HttpResponse,
    web::{Data, Json, Path},
};
use api_utils::context::WyrmContext;
use database::{
    models::{
        feed::Feed,
        webhook::{FeedWebhook, Webhook, WebhookInsertForm, WebhookKind, WebhookUpdateForm},
    },
    newtypes::{FeedId, UserId, WebhookId},
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
    ctx.deny_in_demo()?;

    webhook::validate(&data.kind, data.payload_template.as_deref())
        .map_err(|e| WyrmError::WebhookTemplate(e.to_string()))?;

    // TODO(#372): the requesting user, once the middleware provides it.
    let user_id = UserId(1);
    let webhook = Webhook::create(
        &ctx.db_pool,
        WebhookInsertForm {
            user_id,
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
    path: Path<WebhookId>,
    Json(data): Json<UpdateWebhook>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Webhook>> {
    ctx.deny_in_demo()?;

    // An unchanged template was already validated when the webhook was created,
    // so only re-validate when the caller supplies a new one.
    if let Some(template) = &data.payload_template {
        webhook::validate(&data.kind, Some(template))
            .map_err(|e| WyrmError::WebhookTemplate(e.to_string()))?;
    }

    // TODO(#372): the requesting user, once the middleware provides it.
    let user_id = UserId(1);
    let webhook = Webhook::update(
        &ctx.db_pool,
        WebhookUpdateForm {
            id: path.into_inner(),
            user_id,
            name: data.name,
            url: data.url,
            kind: Some(data.kind),
            payload_template: data.payload_template,
        },
    )
    .await?;
    Ok(Json(webhook))
}

pub async fn delete(path: Path<WebhookId>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Webhook>> {
    ctx.deny_in_demo()?;

    // TODO(#372): the requesting user, once the middleware provides it.
    let user_id = UserId(1);
    let webhook = Webhook::delete(&ctx.db_pool, user_id, path.into_inner()).await?;
    Ok(Json(webhook))
}

pub async fn attach(
    path: Path<(FeedId, WebhookId)>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<HttpResponse> {
    ctx.deny_in_demo()?;

    // TODO(#372): the requesting user, once the middleware provides it.
    let user_id = UserId(1);
    let (feed_id, webhook_id) = path.into_inner();

    // Both must be the user. 404 otherwise.
    Feed::get(&ctx.db_pool, user_id, feed_id).await?;
    Webhook::get(&ctx.db_pool, user_id, webhook_id).await?;

    FeedWebhook::create(&ctx.db_pool, feed_id, webhook_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn detach(
    path: Path<(FeedId, WebhookId)>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<HttpResponse> {
    ctx.deny_in_demo()?;

    // TODO(#372): the requesting user, once the middleware provides it.
    let user_id = UserId(1);
    let (feed_id, webhook_id) = path.into_inner();

    // Feed must be owned by the user. 404 otherwise.
    Feed::get(&ctx.db_pool, user_id, feed_id).await?;

    FeedWebhook::delete(&ctx.db_pool, feed_id, webhook_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
