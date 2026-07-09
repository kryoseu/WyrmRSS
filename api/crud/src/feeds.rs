use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::{
    models::feed::{Feed, FeedInsertForm, FeedUpdateForm},
    newtypes::FeedId,
};
use serde::Deserialize;
use wyrm_utils::result::WyrmResult;

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CreateFeed {
    title: String,
    url: String,
    ttl: i32,
    folder: Option<String>,
    filters: Option<Vec<String>>,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdateFeed {
    title: Option<String>,
    url: Option<String>,
    ttl: Option<i32>,
    /// Absent (None) = keep the current folder;
    /// `null` or blank (Some(None)) = remove the feed from its folder;
    /// a name (Some(Some("name"))) = assign (creating the folder if needed).
    #[serde(default, with = "serde_with::rust::double_option")]
    #[ts(optional, as = "Option<Option<String>>")]
    folder: Option<Option<String>>,
    filters: Option<Vec<String>>,
}

pub async fn create(
    Json(data): Json<CreateFeed>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Feed>> {
    let feed = Feed::create(
        &ctx.db_pool,
        FeedInsertForm {
            title: data.title,
            url: data.url,
            ttl: data.ttl,
            folder_id: None,
            filters: data.filters.map(|v| v.into_iter().map(Some).collect()),
        },
        data.folder.as_deref(),
    )
    .await?;
    Ok(Json(feed))
}

pub async fn update(
    path: Path<FeedId>,
    Json(data): Json<UpdateFeed>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Feed>> {
    let feed = Feed::update(
        &ctx.db_pool,
        FeedUpdateForm {
            id: path.into_inner(),
            title: data.title,
            url: data.url,
            ttl: data.ttl,
            folder_id: None,
            filters: data.filters.map(|v| v.into_iter().map(Some).collect()),
            last_fetched_at: None,
        },
        data.folder.as_ref().map(Option::as_deref),
    )
    .await?;
    Ok(Json(feed))
}

pub async fn delete(path: Path<FeedId>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Feed>> {
    let feed = Feed::delete(&ctx.db_pool, path.into_inner()).await?;
    Ok(Json(feed))
}
