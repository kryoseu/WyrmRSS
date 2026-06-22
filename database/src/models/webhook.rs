use crate::{
    DatabasePool,
    newtypes::{FeedId, WebhookId},
    schema::{feed_webhooks, webhooks},
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[derive(Clone, Debug, DbEnum, Serialize, Deserialize, ts_rs::TS)]
#[ExistingTypePath = "crate::schema::sql_types::WebhookKind"]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case", export)]
pub enum WebhookKind {
    Slack,
    Discord,
    Custom,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::feed_webhooks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(ts_rs::TS)]
#[ts(optional_fields, export)]
/// A join-table row linking a feed to a webhook (many-to-many): a feed can have
/// many webhooks and a webhook can serve many feeds. Rows are managed via
/// [`FeedWebhook::create`]/[`FeedWebhook::delete`] and cascade-deleted when
/// either the feed or the webhook is removed.
pub struct FeedWebhook {
    pub feed_id: FeedId,
    pub webhook_id: WebhookId,
}

#[serde_with::skip_serializing_none]
#[derive(Clone, Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::webhooks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(ts_rs::TS)]
#[ts(optional_fields, export)]
/// A reusable notification destination. A webhook is created independently and
/// attached to any number of feeds via the `feed_webhooks` join table; when a
/// feed gets new posts, the worker POSTs the rendered [`WebhookKind`] payload
/// to its `url`.
pub struct Webhook {
    /// Primary key
    pub id: WebhookId,
    /// Label shown in the UI.
    pub name: String,
    /// Endpoint the payload is POSTed to.
    pub url: String,
    /// Selects the payload format.
    pub kind: WebhookKind,
    /// JSON template rendered for `Custom` webhooks; ignored by the built-ins.
    pub payload_template: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Webhook {
    pub async fn get(pool: &DatabasePool, webhook_id: WebhookId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        webhooks::table
            .find(webhook_id)
            .select(Webhook::as_select())
            .first(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn get_all(pool: &DatabasePool) -> WyrmResult<Vec<Self>> {
        let mut conn = pool.get().await?;
        webhooks::table
            .select(Webhook::as_select())
            .load(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn create(pool: &DatabasePool, form: WebhookInsertForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::insert_into(webhooks::table)
            .values(form)
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn update(pool: &DatabasePool, form: WebhookUpdateForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::update(webhooks::table.find(form.id))
            .set(form)
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn delete(pool: &DatabasePool, webhook_id: WebhookId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::delete(webhooks::table.find(webhook_id))
            .returning(Self::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(WyrmError::from)
    }
}

impl FeedWebhook {
    /// Attaches a webhook to a feed. Idempotent: re-attaching an existing pair
    /// is a no-op rather than a unique-violation error.
    pub async fn create(
        pool: &DatabasePool,
        feed_id: FeedId,
        webhook_id: WebhookId,
    ) -> WyrmResult<()> {
        let mut conn = pool.get().await?;
        diesel::insert_into(feed_webhooks::table)
            .values((
                feed_webhooks::feed_id.eq(feed_id),
                feed_webhooks::webhook_id.eq(webhook_id),
            ))
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    /// Detaches a webhook from a feed. A no-op if the pair isn't attached.
    pub async fn delete(
        pool: &DatabasePool,
        feed_id: FeedId,
        webhook_id: WebhookId,
    ) -> WyrmResult<()> {
        let mut conn = pool.get().await?;
        diesel::delete(
            feed_webhooks::table
                .filter(feed_webhooks::feed_id.eq(feed_id))
                .filter(feed_webhooks::webhook_id.eq(webhook_id)),
        )
        .execute(&mut conn)
        .await?;
        Ok(())
    }
}

#[derive(Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::webhooks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WebhookUpdateForm {
    pub id: WebhookId,
    pub name: Option<String>,
    pub url: Option<String>,
    pub kind: Option<WebhookKind>,
    pub payload_template: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::webhooks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WebhookInsertForm {
    pub name: String,
    pub url: String,
    pub kind: WebhookKind,
    pub payload_template: Option<String>,
}
