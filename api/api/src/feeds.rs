use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::models::feed::Feed;
use utils::result::WyrmResult;

pub async fn get(path: Path<i32>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Feed>> {
    let feed = Feed::get(&ctx.db_pool, path.into_inner()).await?;
    Ok(Json(feed))
}

pub async fn list(ctx: Data<WyrmContext>) -> WyrmResult<Json<Vec<Feed>>> {
    let feeds = Feed::get_all(&ctx.db_pool).await?;
    Ok(Json(feeds))
}
