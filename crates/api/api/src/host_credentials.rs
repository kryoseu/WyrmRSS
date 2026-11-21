use actix_web::web::{Data, Json};
use api_utils::context::WyrmContext;
use database::{models::host_credential::HostCredential, newtypes::UserId};
use wyrm_utils::result::WyrmResult;

pub async fn list(ctx: Data<WyrmContext>) -> WyrmResult<Json<Vec<HostCredential>>> {
    // TODO(#372): the requesting user, once the middleware provides it.
    let user_id = UserId(1);
    let host_credentials = HostCredential::get_all_for_user(&ctx.db_pool, user_id).await?;
    Ok(Json(host_credentials))
}
