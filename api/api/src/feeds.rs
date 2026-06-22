use actix_web::{
    HttpResponse,
    web::{Data, Json, Path},
};
use api_utils::context::WyrmContext;
use database::{models::feed::Feed, newtypes::FeedId};
use std::time::Duration;
use tokio::sync::mpsc::error::TrySendError;
use wyrm_rss::worker::WorkerCommand;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

pub async fn get(path: Path<FeedId>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Feed>> {
    let feed = Feed::get(&ctx.db_pool, path.into_inner()).await?;
    Ok(Json(feed))
}

pub async fn list(ctx: Data<WyrmContext>) -> WyrmResult<Json<Vec<Feed>>> {
    let feeds = Feed::get_all(&ctx.db_pool).await?;
    Ok(Json(feeds))
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

    match tokio::time::timeout(Duration::from_secs(30), rx).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Ok(HttpResponse::GatewayTimeout().finish()),
    }
}
