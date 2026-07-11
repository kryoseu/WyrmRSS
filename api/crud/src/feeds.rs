use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::{
    models::{
        feed::{Feed, FeedInsertForm, FeedUpdateForm},
        feed_icon::FeedIcon,
    },
    newtypes::FeedId,
};
use serde::Deserialize;
use wyrm_rss::{
    discover::{FetchedUrl, fetch_url, resolve_feed_url},
    icon::ensure_feed_icon,
};
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
    // One fetch serves both discovery and the icon lookup below: we try to
    // find a feed from a normal page url (blog homepage, YouTube channel),
    // and if the URL provided already resolves to a feed, we simply pass through.
    let fetched = fetch_url(&ctx.http, &data.url).await;
    let url = resolve_feed_url(&data.url, &fetched)?;
    let feed = Feed::create(
        &ctx.db_pool,
        FeedInsertForm {
            title: data.title,
            url,
            ttl: data.ttl,
            folder: data.folder,
            filters: data.filters.map(|v| v.into_iter().map(Some).collect()),
        },
    )
    .await?;

    // Resolved here when discovery already parsed the feed, so the icon is
    // stored before the frontend refetches the feed list.
    if let Some(parsed) = fetched.parsed_feed() {
        ensure_feed_icon(&ctx.db_pool, &ctx.http, &feed, parsed).await;
    }

    Ok(Json(feed))
}

pub async fn update(
    path: Path<FeedId>,
    Json(data): Json<UpdateFeed>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Feed>> {
    let id = path.into_inner();
    // Only a *changed* URL triggers discovery. The edit form always sends the
    // url field, so an unchanged one must pass through without a network
    // round-trip, otherwise a TTL/title edit would fail while the feed's
    // site happens to be down.
    let feed = Feed::get(&ctx.db_pool, id).await?;
    let (url, fetched) = match data.url {
        Some(u) if u == feed.url => (Some(u), FetchedUrl::None),
        Some(u) => {
            let fetched = fetch_url(&ctx.http, &u).await;
            (Some(resolve_feed_url(&u, &fetched)?), fetched)
        }
        None => (None, FetchedUrl::None),
    };
    let url_changed = url.as_deref().is_some_and(|u| u != feed.url);

    let feed = Feed::update(
        &ctx.db_pool,
        FeedUpdateForm {
            id,
            title: data.title,
            url,
            ttl: data.ttl,
            folder: data.folder,
            filters: data.filters.map(|v| v.into_iter().map(Some).collect()),
            last_fetched_at: None,
        },
    )
    .await?;

    // When feed url has been updated, we need to delete current icon and
    // fetch the new one; right away when discovery already parsed the feed,
    // otherwise at the next poll (the missing row triggers resolution).
    if url_changed {
        FeedIcon::delete(&ctx.db_pool, id).await?;
        if let Some(parsed) = fetched.parsed_feed() {
            ensure_feed_icon(&ctx.db_pool, &ctx.http, &feed, parsed).await;
        }
    }

    Ok(Json(feed))
}

pub async fn delete(path: Path<FeedId>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Feed>> {
    let feed = Feed::delete(&ctx.db_pool, path.into_inner()).await?;
    Ok(Json(feed))
}
