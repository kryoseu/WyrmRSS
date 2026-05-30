use std::time::Duration;

use actix_web::{
    HttpResponse,
    web::{Data, Json, Path},
};
use api_utils::context::WyrmContext;
use database::models::feed::Feed;
use tokio::sync::mpsc::error::TrySendError;
use utils::{error::WyrmError, result::WyrmResult};
use wyrm_rss::worker::WorkerCommand;

pub async fn get(path: Path<i32>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Feed>> {
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
        Err(TrySendError::Full(_)) => return Ok(HttpResponse::Accepted().finish()),
        Err(TrySendError::Closed(_)) => {
            tracing::error!("feed worker channel closed");
            return Err(WyrmError::WorkerError);
        }
        Ok(_) => {}
    }

    match tokio::time::timeout(Duration::from_secs(30), rx).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Ok(HttpResponse::GatewayTimeout().finish()),
    }
}
