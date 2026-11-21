use actix_web::web::{Data, Json};
use api_utils::context::WyrmContext;
use database::{
    models::shortcut::{ShortcutOverride, Shortcuts},
    newtypes::UserId,
};
use wyrm_utils::result::WyrmResult;

pub async fn list(ctx: Data<WyrmContext>) -> WyrmResult<Json<Shortcuts>> {
    // TODO(#372): the requesting user, once the middleware provides it.
    let user_id = UserId(1);
    let overrides = ShortcutOverride::get_all(&ctx.db_pool, user_id).await?;
    let shortcuts = Shortcuts::from_overrides(overrides);
    Ok(Json(shortcuts))
}
