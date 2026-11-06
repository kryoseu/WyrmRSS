use actix_web::web::{Data, Json};
use api_utils::{context::WyrmContext, response::SettingsResponse};
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
    #[serde(default, with = "serde_with::rust::double_option")]
    #[ts(optional, as = "Option<Option<String>>")]
    http_user_agent: Option<Option<String>>,
    #[serde(default, with = "serde_with::rust::double_option")]
    #[ts(optional, as = "Option<Option<i32>>")]
    expire_read_after_days: Option<Option<i32>>,
    #[serde(default, with = "serde_with::rust::double_option")]
    #[ts(optional, as = "Option<Option<i32>>")]
    expire_unread_after_days: Option<Option<i32>>,
    read_delay_secs: Option<i32>,
    rsshub_instance: Option<String>,
}

pub async fn update(
    Json(data): Json<UpdateSettings>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<SettingsResponse>> {
    ctx.deny_in_demo()?;

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
            read_delay_secs: data.read_delay_secs,
            rsshub_instance: data
                .rsshub_instance
                .map(|value| wyrm_rss::rsshub::validate_instance(&value))
                .transpose()?,
        },
    )
    .await?;

    *ctx.runtime_settings.write()? = RuntimeSettings::from(&settings);
    // Interrupt the worker's sleep so it reconfigures immediately; safe to drop
    // since runtime_settings is already updated and the worker reads it on its
    // next iteration.
    let _ = ctx.worker_tx.try_send(WorkerCommand::Reconfigure);

    Ok(Json(SettingsResponse {
        settings,
        run_as_demo: ctx.run_as_demo,
    }))
}

#[cfg(test)]
mod tests {
    use super::UpdateSettings;

    #[test]
    fn partial_settings_distinguish_omitted_null_and_values() {
        let partial: UpdateSettings =
            serde_json::from_str(r#"{"rsshub_instance":"https://example.com/rsshub"}"#).unwrap();
        assert_eq!(partial.http_user_agent, None);
        assert_eq!(partial.expire_read_after_days, None);
        assert_eq!(partial.expire_unread_after_days, None);
        let cleared: UpdateSettings = serde_json::from_str(
            r#"{"http_user_agent":null,"expire_read_after_days":null,"expire_unread_after_days":null}"#,
        ).unwrap();
        assert_eq!(cleared.http_user_agent, Some(None));
        assert_eq!(cleared.expire_read_after_days, Some(None));
        assert_eq!(cleared.expire_unread_after_days, Some(None));
        let set: UpdateSettings = serde_json::from_str(
            r#"{"http_user_agent":"custom","expire_read_after_days":30,"expire_unread_after_days":90}"#,
        ).unwrap();
        assert_eq!(set.http_user_agent, Some(Some("custom".into())));
        assert_eq!(set.expire_read_after_days, Some(Some(30)));
        assert_eq!(set.expire_unread_after_days, Some(Some(90)));
    }
}
