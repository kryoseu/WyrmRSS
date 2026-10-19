use crate::{DatabasePool, newtypes::HostCredentialId, schema::host_credentials};
use chrono::{DateTime, Utc};
use diesel::{Selectable, deserialize::Queryable, prelude::*};
use diesel_async::RunQueryDsl;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use wyrm_utils::{error::WyrmError, result::WyrmResult};

/// How credentials are attached to requests for a host.
/// - `BasicAuth`: `Authorization: Basic`, from `name` (username) and `secret` (password).
/// - `CustomHeader`: a `name: secret` header, e.g. `x-api-key` or `Authorization: Bearer …`.
#[derive(Clone, Debug, DbEnum, Serialize, Deserialize, ts_rs::TS)]
// See the note on `ReadMode` in models/settings.rs: sqlite needs the derive to
// generate its own `HostCredentialKindMapping` rather than reuse a schema type.
#[cfg_attr(
    feature = "postgres",
    ExistingTypePath = "crate::schema::sql_types::HostCredentialKind"
)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case", export)]
pub enum HostCredentialKind {
    BasicAuth,
    CustomHeader,
}

#[serde_with::skip_serializing_none]
#[derive(Clone, Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::host_credentials)]
#[diesel(check_for_backend(crate::Backend))]
#[derive(ts_rs::TS)]
#[ts(optional_fields, export)]
/// Credentials sent with every outbound request (feed polls, discovery, icons)
/// to a matching host.
pub struct HostCredential {
    /// Primary key
    pub id: HostCredentialId,
    /// Host the credentials apply to, as `host:port`. Matched exactly.
    pub host: String,
    /// Selects how `name` and `secret` are sent.
    pub kind: HostCredentialKind,
    /// Username for basic auth, header name for a custom header.
    pub name: String,
    /// Password for basic auth, header value for a custom header.
    /// Write-only: never serialized into API responses.
    #[serde(skip_serializing)]
    #[ts(skip)]
    pub secret: String,
    pub created_at: DateTime<Utc>,
}

impl HostCredential {
    pub async fn get(pool: &DatabasePool, id: HostCredentialId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        host_credentials::table
            .find(id)
            .select(HostCredential::as_select())
            .first(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn get_all(pool: &DatabasePool) -> WyrmResult<Vec<Self>> {
        let mut conn = pool.get().await?;
        host_credentials::table
            .select(HostCredential::as_select())
            .load(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn create(pool: &DatabasePool, form: HostCredentialInsertForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::insert_into(host_credentials::table)
            .values(form)
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn update(pool: &DatabasePool, form: HostCredentialUpdateForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::update(host_credentials::table.find(form.id))
            .set(form)
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn delete(pool: &DatabasePool, id: HostCredentialId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::delete(host_credentials::table.find(id))
            .returning(Self::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(WyrmError::from)
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::host_credentials)]
#[diesel(check_for_backend(crate::Backend))]
pub struct HostCredentialInsertForm {
    pub host: String,
    pub kind: HostCredentialKind,
    pub name: String,
    pub secret: String,
}

#[derive(Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::host_credentials)]
#[diesel(check_for_backend(crate::Backend))]
pub struct HostCredentialUpdateForm {
    pub id: HostCredentialId,
    pub host: Option<String>,
    pub kind: Option<HostCredentialKind>,
    pub name: Option<String>,
    pub secret: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_is_never_serialized() {
        let credential = HostCredential {
            id: HostCredentialId(1),
            host: "feeds.example.com:443".into(),
            kind: HostCredentialKind::BasicAuth,
            name: "reader".into(),
            secret: "hunter2".into(),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&credential).unwrap();

        // The other fields still serialize, so the check below isn't vacuous.
        assert!(json.contains("\"name\":\"reader\""));
        assert!(!json.contains("secret"));
        assert!(!json.contains("hunter2"));
    }

    #[cfg(feature = "postgres")]
    #[tokio::test]
    async fn create_update_delete_round_trip() {
        let pool = crate::setup_test_db().await;
        // Unique host per run: the test database is shared across concurrent tests.
        let host = format!(
            "{}.example.com:443",
            Utc::now()
                .timestamp_nanos_opt()
                .expect("timestamp in range")
        );
        let insert_form = || HostCredentialInsertForm {
            host: host.clone(),
            kind: HostCredentialKind::BasicAuth,
            name: "reader".into(),
            secret: "hunter2".into(),
        };

        let created = HostCredential::create(&pool, insert_form())
            .await
            .expect("create should succeed");
        assert!(
            matches!(
                HostCredential::create(&pool, insert_form()).await,
                Err(WyrmError::Database(
                    wyrm_utils::error::DatabaseError::UniqueViolation(_)
                ))
            ),
            "a second credential for the same host should conflict"
        );

        // A partial update leaves the other fields untouched.
        let updated = HostCredential::update(
            &pool,
            HostCredentialUpdateForm {
                id: created.id,
                host: None,
                kind: None,
                name: None,
                secret: Some("correct horse".into()),
            },
        )
        .await
        .expect("update should succeed");
        assert_eq!(updated.secret, "correct horse");
        assert_eq!(updated.name, "reader");
        assert_eq!(updated.host, host);

        let fetched = HostCredential::get(&pool, created.id)
            .await
            .expect("get should succeed");
        assert_eq!(fetched.secret, "correct horse");

        HostCredential::delete(&pool, created.id)
            .await
            .expect("delete should succeed");
        assert!(
            HostCredential::get(&pool, created.id).await.is_err(),
            "deleted credential should be gone"
        );
    }
}
