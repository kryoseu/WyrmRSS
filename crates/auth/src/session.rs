use actix_web::{
    HttpRequest,
    cookie::{Cookie, SameSite, time::Duration as CookieDuration},
    http::header,
};
use chrono::{DateTime, TimeDelta, Utc};
use database::{
    DatabasePool,
    models::session::{Session, SessionInsertForm},
};
use sha2::{Digest, Sha256};
use url::Url;
use wyrm_utils::{error::AuthError, result::WyrmResult};

/// How long a session survives without requests.
/// User activity extends it, at most once per [`SESSION_REFRESH_INTERVAL`].
pub(crate) const SESSION_IDLE_TIMEOUT: TimeDelta = TimeDelta::hours(12);
/// How long after its last refresh a session goes before its expiry is extended again.
/// Requests within this window only read the session, avoiding a DB write per request.
const SESSION_REFRESH_INTERVAL: TimeDelta = TimeDelta::minutes(5);

/// Wyrm's own login sessions: creates, authenticates and revokes them,
/// whichever method signed the user in.
pub struct SessionManager {
    /// Wyrm's own origin (`scheme://host[:port]` of `public_url`), used by `check_csrf`:
    /// a cookie-authenticated write whose `Origin` header names another site is rejected,
    /// so a page elsewhere can't use the browser's session cookie to make changes.
    pub origin: String,
    /// Served over HTTPS: session cookie gets `Secure` and the `__Host-` prefix.
    pub secure: bool,
}

/// Outcome of `authenticate`.
pub struct Authenticated {
    /// (Re-)issued session cookie: set when the session was just created or its expiry extended.
    pub cookie: Option<Cookie<'static>>,
}

impl SessionManager {
    pub async fn create(&self, pool: &DatabasePool) -> WyrmResult<Authenticated> {
        let token = generate_session_token();
        let now = Utc::now();
        let expires_at = now + SESSION_IDLE_TIMEOUT;

        // Housekeeping only: expired sessions can't authenticate, so a failure mustn't block login.
        if let Err(error) = Session::delete_expired(pool).await {
            tracing::warn!(%error, "could not delete expired sessions");
        }

        Session::create(
            pool,
            SessionInsertForm {
                token_hash: token_hash(&token),
                expires_at,
                last_seen_at: now,
            },
        )
        .await?;

        Ok(Authenticated {
            cookie: Some(self.cookie(token, expires_at)),
        })
    }

    pub async fn authenticate(
        &self,
        pool: &DatabasePool,
        req: &HttpRequest,
    ) -> WyrmResult<Option<Authenticated>> {
        let Some(cookie) = req.cookie(self.cookie_name()) else {
            return Ok(None);
        };

        let token_hash = token_hash(cookie.value());
        let Some(session) = Session::find_active(pool, &token_hash).await? else {
            return Ok(None);
        };

        // Refreshed recently: valid as is, no write needed.
        let now = Utc::now();
        if now - session.last_seen_at < SESSION_REFRESH_INTERVAL {
            return Ok(Some(Authenticated { cookie: None }));
        }

        // None if it expired or was revoked between the read and this write.
        let Some(session) = Session::refresh(pool, &token_hash, now + SESSION_IDLE_TIMEOUT).await?
        else {
            return Ok(None);
        };

        Ok(Some(Authenticated {
            cookie: Some(self.cookie(cookie.value().to_owned(), session.expires_at)),
        }))
    }

    pub async fn revoke(
        &self,
        pool: &DatabasePool,
        req: &HttpRequest,
    ) -> WyrmResult<Cookie<'static>> {
        if let Some(cookie) = req.cookie(self.cookie_name()) {
            Session::delete(pool, &token_hash(cookie.value())).await?;
        }
        // Already expired, so the browser deletes it.
        Ok(self.cookie(String::new(), DateTime::UNIX_EPOCH))
    }

    pub fn check_csrf(&self, req: &HttpRequest) -> WyrmResult<()> {
        if req
            .headers()
            .get("x-wyrm-csrf")
            .and_then(|v| v.to_str().ok())
            != Some("1")
            || req
                .headers()
                .get(header::ORIGIN)
                .is_some_and(|origin| origin.to_str().ok() != Some(&self.origin))
        {
            return Err(AuthError::CsrfCheckFailed.into());
        }
        Ok(())
    }

    pub(crate) fn from_public_url(url: &Url) -> Self {
        Self {
            origin: url.origin().ascii_serialization(),
            secure: url.scheme() == "https",
        }
    }

    fn cookie(&self, token: String, expires_at: DateTime<Utc>) -> Cookie<'static> {
        let ttl = (expires_at - Utc::now()).to_std().unwrap_or_default();

        Cookie::build(self.cookie_name(), token)
            .path("/")
            .http_only(true)
            .secure(self.secure)
            .same_site(SameSite::Lax)
            .max_age(CookieDuration::seconds(ttl.as_secs() as i64))
            .finish()
    }

    pub(crate) fn cookie_name(&self) -> &'static str {
        if self.secure {
            "__Host-wyrm_session"
        } else {
            "wyrm_session"
        }
    }
}

pub(crate) fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("OS random source unavailable");
    hex::encode(bytes)
}

pub(crate) fn token_hash(token: &str) -> String {
    hex::encode(Sha256::digest(token))
}
