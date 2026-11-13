use crate::{DatabasePool, schema::sessions};
use chrono::{DateTime, Utc};
use diesel::dsl::case_when;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
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
    /// Create a session and revoke its predecessor atomically. Only hashes of
    /// the random browser tokens are persisted.
    pub async fn create(
        pool: &DatabasePool,
        form: SessionInsertForm,
        previous_hash: Option<&str>,
    ) -> WyrmResult<()> {
        let mut conn = pool.get().await?;
        conn.transaction(async |conn| {
            diesel::delete(sessions::table.filter(sessions::expires_at.le(form.last_seen_at)))
                .execute(conn)
                .await?;
            if let Some(previous) = previous_hash {
                diesel::delete(sessions::table.find(previous))
                    .execute(conn)
                    .await?;
            }
            diesel::insert_into(sessions::table)
                .values(&form)
                .execute(conn)
                .await?;
            Ok(())
        })
        .await
    }

    /// Authenticate and record activity in one update. Expired or revoked
    /// sessions cannot be revived, and concurrent requests never shorten expiry.
    pub async fn refresh(
        pool: &DatabasePool,
        token_hash: &str,
        now: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> WyrmResult<Option<Self>> {
        let mut conn = pool.get().await?;
        diesel::update(
            sessions::table
                .find(token_hash)
                .filter(sessions::expires_at.gt(now)),
        )
        .set((
            sessions::last_seen_at.eq(case_when(
                sessions::last_seen_at.gt(now),
                sessions::last_seen_at,
            )
            .otherwise(now)),
            sessions::expires_at.eq(case_when(
                sessions::expires_at.gt(expires_at),
                sessions::expires_at,
            )
            .otherwise(expires_at)),
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
            .await?;
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

    fn insert_form(label: &str, now: DateTime<Utc>) -> SessionInsertForm {
        SessionInsertForm {
            token_hash: format!("{label}-{}", Utc::now().timestamp_nanos_opt().unwrap()),
            expires_at: now + TimeDelta::hours(12),
            last_seen_at: now,
        }
    }

    #[tokio::test]
    async fn activity_extends_session_but_cannot_revive_expired_sessions() {
        let pool = crate::setup_test_db().await;
        let now = DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap();
        let original = insert_form("activity", now);
        let hash = original.token_hash.clone();
        Session::create(&pool, original, None).await.unwrap();

        let active = now + TimeDelta::hours(11);
        let expiry = active + TimeDelta::hours(12);
        let renewed = Session::refresh(&pool, &hash, active, expiry)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(renewed.last_seen_at, active);
        assert_eq!(renewed.expires_at, expiry);
        let delayed = Session::refresh(&pool, &hash, now, now + TimeDelta::hours(12))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(delayed.last_seen_at, active);
        assert_eq!(delayed.expires_at, expiry);
        assert!(
            Session::refresh(&pool, &hash, expiry, expiry + TimeDelta::hours(12))
                .await
                .unwrap()
                .is_none()
        );
        Session::delete(&pool, &hash).await.unwrap();
    }

    #[tokio::test]
    async fn rotation_is_atomic_and_logout_revokes_the_session() {
        let pool = crate::setup_test_db().await;
        let now = Utc::now();
        let original = insert_form("original", now);
        let replacement = insert_form("replacement", now);
        let old_hash = original.token_hash.clone();
        let new_hash = replacement.token_hash.clone();
        let replacement_again = || SessionInsertForm {
            token_hash: new_hash.clone(),
            expires_at: replacement.expires_at,
            last_seen_at: replacement.last_seen_at,
        };
        Session::create(&pool, original, None).await.unwrap();
        Session::create(&pool, replacement_again(), None)
            .await
            .unwrap();
        // A failed replacement must leave the previous session valid.
        assert!(
            Session::create(&pool, replacement_again(), Some(&old_hash))
                .await
                .is_err()
        );
        assert!(
            Session::refresh(&pool, &old_hash, now, now + TimeDelta::hours(12))
                .await
                .unwrap()
                .is_some()
        );
        Session::delete(&pool, &new_hash).await.unwrap();
        Session::create(&pool, replacement_again(), Some(&old_hash))
            .await
            .unwrap();
        assert!(
            Session::refresh(&pool, &old_hash, now, now + TimeDelta::hours(12))
                .await
                .unwrap()
                .is_none()
        );
        Session::delete(&pool, &new_hash).await.unwrap();
        assert!(
            Session::refresh(&pool, &new_hash, now, now + TimeDelta::hours(12))
                .await
                .unwrap()
                .is_none()
        );
    }
}
