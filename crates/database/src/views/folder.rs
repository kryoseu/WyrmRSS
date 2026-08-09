use crate::{
    DatabasePool,
    models::folder::Folder,
    newtypes::FeedId,
    schema::{feeds, folders},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

/// Returns a hashmap containing all folders by feedId. Standalone feeds are absent
pub async fn all_by_feed(pool: &DatabasePool) -> WyrmResult<HashMap<FeedId, Folder>> {
    let mut conn = pool.get().await?;
    let rows: Vec<(FeedId, Folder)> = feeds::table
        .inner_join(folders::table)
        .select((feeds::id, Folder::as_select()))
        .load(&mut conn)
        .await
        .map_err(WyrmError::from)?;
    Ok(rows.into_iter().collect())
}
