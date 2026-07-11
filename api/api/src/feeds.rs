use actix_web::{
    HttpResponse,
    http::header,
    web::{Data, Json, Path},
};
use api_utils::context::WyrmContext;
use database::{
    models::{feed::Feed, feed_icon::FeedIcon},
    newtypes::FeedId,
    views::{self, feed::FeedView},
};
use std::time::Duration;
use tokio::sync::mpsc::error::TrySendError;
use wyrm_rss::worker::WorkerCommand;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

pub async fn get(path: Path<FeedId>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Feed>> {
    let feed = Feed::get(&ctx.db_pool, path.into_inner()).await?;
    Ok(Json(feed))
}

pub async fn list(ctx: Data<WyrmContext>) -> WyrmResult<Json<Vec<FeedView>>> {
    let feeds = views::feed::list(&ctx.db_pool).await?;
    Ok(Json(feeds))
}

pub async fn icon(path: Path<FeedId>, ctx: Data<WyrmContext>) -> WyrmResult<HttpResponse> {
    let icon = FeedIcon::get(&ctx.db_pool, path.into_inner()).await?;
    let Some(FeedIcon {
        data: Some(data),
        content_type,
        ..
    }) = icon
    else {
        return Ok(HttpResponse::NotFound().finish());
    };
    Ok(HttpResponse::Ok()
        .content_type(content_type.unwrap_or_else(|| "image/x-icon".to_string()))
        // Icons rarely change; spare the repeat requests as post lists render.
        .insert_header((header::CACHE_CONTROL, "max-age=86400"))
        .body(data))
}

pub async fn poll(ctx: Data<WyrmContext>) -> WyrmResult<HttpResponse> {
    // channel through which worker signals completion
    let (tx, rx) = tokio::sync::oneshot::channel();

    match ctx.worker_tx.try_send(WorkerCommand::PollFeeds(tx)) {
        // channel is full, meaning a poll is in progress already
        Err(TrySendError::Full(_)) => return Ok(HttpResponse::Accepted().finish()),
        Err(TrySendError::Closed(_)) => {
            let err = "feed worker channel closed";
            return Err(WyrmError::WorkerError(err.to_string()));
        }
        Ok(_) => {}
    }

    // Wait for the configured http timeout.
    let http_timeout = ctx.runtime_settings.read()?.http_timeout;
    match tokio::time::timeout(Duration::from_secs(http_timeout as u64), rx).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Ok(HttpResponse::GatewayTimeout().finish()),
    }
}
