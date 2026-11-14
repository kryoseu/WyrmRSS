use crate::{DatabasePool, schema::sessions};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[derive(Queryable, Selectable)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(crate::Backend))]
pub struct Session {
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

impl Session {
    pub async fn create(pool: &DatabasePool, form: SessionInsertForm) -> WyrmResult<()> {
        let mut conn = pool.get().await?;
        diesel::insert_into(sessions::table)
            .values(&form)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    /// The session for `token_hash`, if it exists and hasn't expired.
    pub async fn find_active(pool: &DatabasePool, token_hash: &str) -> WyrmResult<Option<Self>> {
        let mut conn = pool.get().await?;
        sessions::table
            .find(token_hash)
            .filter(sessions::expires_at.gt(Utc::now()))
            .select(Self::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(WyrmError::from)
    }

    /// Removes sessions past their expiry; returns how many were deleted.
    pub async fn delete_expired(pool: &DatabasePool) -> WyrmResult<usize> {
        let mut conn = pool.get().await?;
        let deleted = diesel::delete(sessions::table.filter(sessions::expires_at.le(Utc::now())))
            .execute(&mut conn)
            .await?;
        Ok(deleted)
    }

    /// Records activity on an active session: sets `last_seen_at` to now and extends `expires_at`.
    /// Expired or revoked sessions can't be revived.
    pub async fn refresh(
        pool: &DatabasePool,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> WyrmResult<Option<Self>> {
        let mut conn = pool.get().await?;
        let now = Utc::now();

        diesel::update(
            sessions::table
                .find(token_hash)
                .filter(sessions::expires_at.gt(now)),
        )
        .set((
            sessions::last_seen_at.eq(now),
            sessions::expires_at.eq(expires_at),
        ))
        .returning(Self::as_returning())
        .get_result(&mut conn)
        .await
        .optional()
        .map_err(WyrmError::from)
    }

    pub async fn delete(pool: &DatabasePool, token_hash: &str) -> WyrmResult<()> {
        let mut conn = pool.get().await?;
        diesel::delete(sessions::table.find(token_hash))
            .execute(&mut conn)
            .await
            .map_err(WyrmError::from)?;
        Ok(())
    }
}

#[derive(Default, Insertable)]
#[diesel(table_name = crate::schema::sessions)]
#[diesel(check_for_backend(crate::Backend))]
pub struct SessionInsertForm {
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[cfg(all(test, feature = "postgres"))]
mod tests {
    use super::*;
    use chrono::TimeDelta;
    use wyrm_utils::error::DatabaseError;

    /// Unique per call: the test database is shared across concurrent tests.
    fn unique_token_hash(label: &str) -> String {
        format!(
            "{label}-{}",
            Utc::now()
                .timestamp_nanos_opt()
                .expect("timestamp in range")
        )
    }

    /// Whole seconds, so values compare equal after a database round trip.
    fn hours_from_now(hours: i64) -> DateTime<Utc> {
        let now = DateTime::from_timestamp(Utc::now().timestamp(), 0).expect("timestamp in range");
        now + TimeDelta::hours(hours)
    }

    fn insert_form(token_hash: &str, expires_at: DateTime<Utc>) -> SessionInsertForm {
        SessionInsertForm {
            token_hash: token_hash.to_owned(),
            expires_at,
            last_seen_at: hours_from_now(0),
        }
    }

    #[tokio::test]
    async fn create_refresh_delete_round_trip() {
        let pool = crate::setup_test_db().await;
        let token_hash = unique_token_hash("round-trip");
        let expires_at = hours_from_now(1);

        Session::create(&pool, insert_form(&token_hash, expires_at))
            .await
            .expect("create should succeed");
        assert!(
            matches!(
                Session::create(&pool, insert_form(&token_hash, expires_at)).await,
                Err(WyrmError::Database(DatabaseError::UniqueViolation(_)))
            ),
            "a second session with the same token hash should conflict"
        );

        let found = Session::find_active(&pool, &token_hash)
            .await
            .expect("find should succeed")
            .expect("active session should be found");
        assert_eq!(found.expires_at, expires_at);

        let extended = hours_from_now(12);
        let refreshed = Session::refresh(&pool, &token_hash, extended)
            .await
            .expect("refresh should succeed")
            .expect("active session should refresh");
        assert_eq!(refreshed.expires_at, extended);

        Session::delete(&pool, &token_hash)
            .await
            .expect("delete should succeed");
        assert!(
            Session::find_active(&pool, &token_hash)
                .await
                .expect("find should succeed")
                .is_none(),
            "deleted session should be gone"
        );

        // Logging out of an already purged session must still succeed.
        Session::delete(&pool, &token_hash)
            .await
            .expect("deleting a missing session should succeed");
    }

    #[tokio::test]
    async fn expired_sessions_are_not_found_not_refreshed_and_get_purged() {
        let pool = crate::setup_test_db().await;
        let expired = unique_token_hash("expired");
        let active = unique_token_hash("active");

        Session::create(&pool, insert_form(&expired, hours_from_now(-1)))
            .await
            .expect("create should succeed");
        Session::create(&pool, insert_form(&active, hours_from_now(1)))
            .await
            .expect("create should succeed");

        assert!(
            Session::find_active(&pool, &expired)
                .await
                .expect("find should succeed")
                .is_none(),
            "expired session should not be found"
        );
        assert!(
            Session::refresh(&pool, &expired, hours_from_now(12))
                .await
                .expect("refresh should succeed")
                .is_none(),
            "expired session should not be revived"
        );

        Session::delete_expired(&pool)
            .await
            .expect("delete_expired should succeed");

        // Other tests share the table, so count only this test's rows.
        let mut conn = pool.get().await.expect("should get conn");
        let remaining: i64 = sessions::table
            .filter(sessions::token_hash.eq_any([&expired, &active]))
            .count()
            .get_result(&mut conn)
            .await
            .expect("count should succeed");
        assert_eq!(remaining, 1, "only the expired session should be purged");
        assert!(
            Session::find_active(&pool, &active)
                .await
                .expect("find should succeed")
                .is_some(),
            "active session should survive the purge"
        );

        Session::delete(&pool, &active)
            .await
            .expect("delete should succeed");
    }
}
