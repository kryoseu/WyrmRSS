use crate::{
    DatabasePool,
    models::webhook::Webhook,
    newtypes::FeedId,
    schema::{feed_webhooks, webhooks},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use std::collections::HashMap;
use wyrm_utils::result::WyrmResult;

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FeedWebhookView {
    #[serde(flatten)]
    webhook: Webhook,
    attached: bool,
}

/// Returns a hashmap containing all webhooks by feedId
pub async fn all_by_feed(pool: &DatabasePool) -> WyrmResult<HashMap<FeedId, Vec<Webhook>>> {
    let mut conn = pool.get().await?;
    let rows: Vec<(FeedId, Webhook)> = feed_webhooks::table
        .inner_join(webhooks::table)
        .select((feed_webhooks::feed_id, Webhook::as_select()))
        .load(&mut conn)
        .await?;

    let mut map: HashMap<FeedId, Vec<Webhook>> = HashMap::new();
    for (feed_id, webhook) in rows {
        map.entry(feed_id).or_default().push(webhook);
    }
    Ok(map)
}

/// All webhooks, with an attached flag indicating which webhooks are
/// attached to the given feed.
///
/// When editting a feed, we need to display the list of all webhooks
/// available that can be attached, but also show which ones already are.
/// This call allows everything to be returned in one call.
pub async fn list_for_feed(
    pool: &DatabasePool,
    feed_id: FeedId,
) -> WyrmResult<Vec<FeedWebhookView>> {
    let mut conn = pool.get().await?;
    let rows: Vec<(Webhook, bool)> = webhooks::table
        .left_join(
            feed_webhooks::table.on(feed_webhooks::webhook_id
                .eq(webhooks::id)
                .and(feed_webhooks::feed_id.nullable().eq(feed_id))),
        )
        .select((
            Webhook::as_select(),
            feed_webhooks::feed_id.nullable().is_not_null(),
        ))
        .load(&mut conn)
        .await?;

    Ok(rows
        .into_iter()
        .map(|(webhook, attached)| FeedWebhookView { webhook, attached })
        .collect())
}
