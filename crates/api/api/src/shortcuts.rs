use actix_web::web::{Data, Json};
use api_utils::context::WyrmContext;
use database::models::shortcut::{ShortcutOverride, Shortcuts};
use wyrm_utils::result::WyrmResult;

pub async fn list(ctx: Data<WyrmContext>) -> WyrmResult<Json<Shortcuts>> {
    let overrides = ShortcutOverride::get_all(&ctx.db_pool).await?;
    let shortcuts = Shortcuts::from_overrides(overrides);
    Ok(Json(shortcuts))
}
