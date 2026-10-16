use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::models::shortcut::{
    ShortcutAction,
    ShortcutOverride,
    ShortcutOverrideInsertForm,
    Shortcuts,
};
use serde::Deserialize;
use wyrm_utils::{
    error::{DatabaseError, WyrmError},
    result::WyrmResult,
};

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdateShortcut {
    key: String,
}

pub async fn update(
    path: Path<String>,
    Json(data): Json<UpdateShortcut>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Shortcuts>> {
    ctx.deny_in_demo()?;

    let shortcut_action = path.into_inner();

    // User sent an unknown shortcut action. 404 like any missing resource.
    let action = shortcut_action
        .parse::<ShortcutAction>()
        .map_err(|_| WyrmError::Database(DatabaseError::NotFound))?;

    let key = data.key;

    if key.trim().is_empty() {
        return Err(WyrmError::InvalidInput("key cannot be empty".into()));
    }

    let current = Shortcuts::from_overrides(ShortcutOverride::get_all(&ctx.db_pool).await?);

    // Re-saving an action's own key is fine, only another action's key conflicts.
    if let Some(owner) = current.action_for_key(&key)
        && owner != action
    {
        return Err(WyrmError::Database(DatabaseError::Conflict(format!(
            "key already bound to {owner:?}"
        ))));
    }

    ShortcutOverride::create(
        &ctx.db_pool,
        ShortcutOverrideInsertForm {
            shortcut_action,
            key,
        },
    )
    .await?;

    let overrides = ShortcutOverride::get_all(&ctx.db_pool).await?;

    Ok(Json(Shortcuts::from_overrides(overrides)))
}

pub async fn delete(path: Path<String>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Shortcuts>> {
    ctx.deny_in_demo()?;

    let action = path.into_inner();

    // User sent an unknown shortcut action. 404 like any missing resource.
    action
        .parse::<ShortcutAction>()
        .map_err(|_| WyrmError::Database(DatabaseError::NotFound))?;

    ShortcutOverride::delete(&ctx.db_pool, action.as_str()).await?;

    let overrides = ShortcutOverride::get_all(&ctx.db_pool).await?;

    let shortcuts = Shortcuts::from_overrides(overrides);

    Ok(Json(shortcuts))
}
