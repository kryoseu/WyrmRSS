use actix_web::web::{Data, Json};
use api_utils::context::WyrmContext;
use database::models::host_credential::HostCredential;
use wyrm_utils::result::WyrmResult;

pub async fn list(ctx: Data<WyrmContext>) -> WyrmResult<Json<Vec<HostCredential>>> {
    let host_credentials = HostCredential::get_all(&ctx.db_pool).await?;
    Ok(Json(host_credentials))
}
