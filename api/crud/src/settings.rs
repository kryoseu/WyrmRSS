use actix_web::web::{Data, Json};
use api_utils::context::WyrmContext;
use database::{
    models::settings::{ReadMode, Settings, SettingsUpdateForm},
    utils::settings::RuntimeSettings,
};
use serde::Deserialize;
use wyrm_rss::worker::WorkerCommand;
use wyrm_utils::result::WyrmResult;

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdateSettings {
    page_size: Option<i32>,
    feed_poll_interval_secs: Option<i32>,
    read_mode: Option<ReadMode>,
    http_timeout: Option<i32>,
    http_connect_timeout: Option<i32>,
    http_retries: Option<i32>,
    http_user_agent: Option<String>,
    expire_read_after_days: Option<i32>,
    expire_unread_after_days: Option<i32>,
}

pub async fn update(
    Json(data): Json<UpdateSettings>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Settings>> {
    let settings = Settings::update(
        &ctx.db_pool,
        SettingsUpdateForm {
            page_size: data.page_size,
            feed_poll_interval_secs: data.feed_poll_interval_secs,
            read_mode: data.read_mode,
            http_timeout: data.http_timeout,
            http_connect_timeout: data.http_connect_timeout,
            http_retries: data.http_retries,
            http_user_agent: data.http_user_agent,
            expire_read_after_days: data.expire_read_after_days,
            expire_unread_after_days: data.expire_unread_after_days,
        },
    )
    .await?;

    *ctx.runtime_settings.write()? = RuntimeSettings::from(&settings);
    // Interrupt the worker's sleep so it reconfigures immediately; safe to drop
    // since runtime_settings is already updated and the worker reads it on its
    // next iteration.
    let _ = ctx.worker_tx.try_send(WorkerCommand::Reconfigure);
    Ok(Json(settings))
}
