use crate::{DatabasePool, schema::settings};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use wyrm_utils::{error::WyrmError, result::WyrmResult};

/// Controls when posts are marked as read.
/// - `OnOpen`: automatically marked read when opened in the reader.
/// - `Manually`: only marked read via the toggle button.
/// - `Disabled`: read state is never updated; all posts appear as read.
#[derive(Clone, Debug, DbEnum, Serialize, Deserialize, ts_rs::TS)]
#[ExistingTypePath = "crate::schema::sql_types::ReadMode"]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case", export)]
pub enum ReadMode {
    OnOpen,
    Manually,
    Disabled,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(ts_rs::TS)]
#[ts(optional_fields, export)]
pub struct Settings {
    /// Always `true`. This column is the primary key, and the `only_one_row`
    /// CHECK pins it to `true` — so the table can hold at most one row.
    #[ts(skip)]
    #[serde(skip)]
    pub is_singleton: bool,
    /// Number of posts returned per page when listing.
    pub page_size: i32,
    /// How often feeds are polled for new posts, in seconds.
    pub feed_poll_interval_secs: i32,
    /// Total timeout for a feed HTTP request, in seconds.
    pub http_timeout: i32,
    /// Timeout for establishing the connection to a feed host, in seconds.
    pub http_connect_timeout: i32,
    /// Number of times a failed feed request is retried.
    pub http_retries: i32,
    /// Custom `User-Agent` header sent when fetching feeds; `None` uses the
    /// default.
    pub http_user_agent: Option<String>,
    /// When posts get marked as read (see [`ReadMode`]).
    pub read_mode: ReadMode,
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
    pub read_mode: Option<ReadMode>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup_test_db;

    /// Settings is a shared singleton, so this snapshots the row, applies an
    /// update, restores the original, and only then asserts — a failed
    /// assertion can't leave the dev database dirty.
    #[tokio::test]
    async fn update_changes_only_given_fields() {
        let pool = setup_test_db().await;

        let original = Settings::get(&pool).await.expect("should read settings");
        assert!(original.is_singleton);

        // Change a couple of fields; leave the rest as `None`.
        let updated = Settings::update(
            &pool,
            SettingsUpdateForm {
                page_size: Some(original.page_size + 1),
                feed_poll_interval_secs: None,
                http_timeout: None,
                http_connect_timeout: None,
                http_retries: None,
                http_user_agent: Some("wyrm-test/1.0".to_string()),
                read_mode: Some(ReadMode::Disabled),
            },
        )
        .await
        .expect("update should succeed");

        // `http_user_agent` uses `treat_none_as_null`, so `None` clears it.
        let cleared = Settings::update(
            &pool,
            SettingsUpdateForm {
                page_size: None,
                feed_poll_interval_secs: None,
                http_timeout: None,
                http_connect_timeout: None,
                http_retries: None,
                http_user_agent: None,
                read_mode: None,
            },
        )
        .await
        .expect("update should succeed");

        // Restore before asserting
        Settings::update(
            &pool,
            SettingsUpdateForm {
                page_size: Some(original.page_size),
                feed_poll_interval_secs: Some(original.feed_poll_interval_secs),
                http_timeout: Some(original.http_timeout),
                http_connect_timeout: Some(original.http_connect_timeout),
                http_retries: Some(original.http_retries),
                http_user_agent: original.http_user_agent.clone(),
                read_mode: Some(original.read_mode.clone()),
            },
        )
        .await
        .expect("restore should succeed");

        // Given fields changed...
        assert_eq!(updated.page_size, original.page_size + 1);
        assert_eq!(updated.http_user_agent.as_deref(), Some("wyrm-test/1.0"));
        assert!(matches!(updated.read_mode, ReadMode::Disabled));
        // ...while `None` fields were left untouched.
        assert_eq!(
            updated.feed_poll_interval_secs,
            original.feed_poll_interval_secs
        );
        // `None` on a `treat_none_as_null` column writes NULL...
        assert_eq!(cleared.http_user_agent, None);
        // ...but a plain `None` field is still left untouched.
        assert_eq!(cleared.page_size, original.page_size + 1);
    }
}
