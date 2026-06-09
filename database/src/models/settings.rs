use crate::{DatabasePool, schema::settings};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(ts_rs::TS)]
#[ts(optional_fields, export)]
pub struct Settings {
    #[ts(skip)]
    #[serde(skip)]
    pub is_singleton: bool,
    pub page_size: i32,
    pub feed_poll_interval_secs: i32,
    pub http_timeout: i32,
    pub http_connect_timeout: i32,
    pub http_retries: i32,
    pub http_user_agent: Option<String>,
}

impl Settings {
    pub async fn get(pool: &DatabasePool) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        settings::table
            .select(Settings::as_select())
            .first(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn update(pool: &DatabasePool, form: SettingsUpdateForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::update(settings::table)
            .set(form)
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SettingsUpdateForm {
    pub page_size: Option<i32>,
    pub feed_poll_interval_secs: Option<i32>,
    pub http_timeout: Option<i32>,
    pub http_connect_timeout: Option<i32>,
    pub http_retries: Option<i32>,
    #[diesel(treat_none_as_null = true)]
    pub http_user_agent: Option<String>,
}
