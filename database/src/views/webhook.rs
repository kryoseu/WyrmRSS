use crate::{
    DatabasePool,
    models::webhook::Webhook,
    schema::{feed_webhooks, webhooks},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use wyrm_utils::result::WyrmResult;

/// Returns a hashmap containing all webhooks by feedId
pub async fn all_by_feed(pool: &DatabasePool) -> WyrmResult<HashMap<i32, Vec<Webhook>>> {
    let mut conn = pool.get().await?;
    let rows: Vec<(i32, Webhook)> = feed_webhooks::table
        .inner_join(webhooks::table)
        .select((feed_webhooks::feed_id, Webhook::as_select()))
        .load(&mut conn)
        .await?;

    let mut map: HashMap<i32, Vec<Webhook>> = HashMap::new();
    for (feed_id, webhook) in rows {
        map.entry(feed_id).or_default().push(webhook);
    }
    Ok(map)
}

/// All webhooks attached to a single feed.
pub async fn for_feed(pool: &DatabasePool, feed_id: i32) -> WyrmResult<Vec<Webhook>> {
    let mut conn = pool.get().await?;
    let webhooks = feed_webhooks::table
        .inner_join(webhooks::table)
        .filter(feed_webhooks::feed_id.eq(feed_id))
        .select(Webhook::as_select())
        .load(&mut conn)
        .await?;
    Ok(webhooks)
}
