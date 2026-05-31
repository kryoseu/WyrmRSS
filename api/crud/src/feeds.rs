use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::models::feed::{Feed, FeedInsertForm, FeedUpdateForm};
use serde::Deserialize;
use utils::result::WyrmResult;

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CreateFeed {
    title: String,
    url: String,
    ttl: i32,
    tag: Option<String>,
    tag_color: Option<String>,
    url_filter: Option<Vec<String>>,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdateFeed {
    title: Option<String>,
    url: Option<String>,
    ttl: Option<i32>,
    tag: Option<String>,
    tag_color: Option<String>,
    url_filter: Option<Vec<String>>,
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
            tag: data.tag,
            tag_color: data.tag_color,
            url_filter: data.url_filter.map(|v| v.into_iter().map(Some).collect()),
        },
    )
    .await?;
    Ok(Json(feed))
}

pub async fn update(
    path: Path<i32>,
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
            tag: data.tag,
            tag_color: data.tag_color,
            url_filter: data.url_filter.map(|v| v.into_iter().map(Some).collect()),
            last_fetched_at: None,
        },
    )
    .await?;
    Ok(Json(feed))
}

pub async fn delete(path: Path<i32>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Feed>> {
    let feed = Feed::delete(&ctx.db_pool, path.into_inner()).await?;
    Ok(Json(feed))
}
