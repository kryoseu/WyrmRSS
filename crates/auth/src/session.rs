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
        if !needs_refresh(session.last_seen_at, now) {
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

/// True once the session's last refresh is at least [`SESSION_REFRESH_INTERVAL`] old.
fn needs_refresh(last_seen_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    now - last_seen_at >= SESSION_REFRESH_INTERVAL
}

pub(crate) fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("OS random source unavailable");
    hex::encode(bytes)
}

pub(crate) fn token_hash(token: &str) -> String {
    hex::encode(Sha256::digest(token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;
    use wyrm_utils::error::WyrmError;

    fn manager(public_url: &str) -> SessionManager {
        SessionManager::from_public_url(&Url::parse(public_url).expect("valid url"))
    }

    fn is_csrf_failure(result: WyrmResult<()>) -> bool {
        matches!(result, Err(WyrmError::Auth(AuthError::CsrfCheckFailed)))
    }

    #[test]
    fn needs_refresh_only_once_the_refresh_interval_has_passed() {
        let now = Utc::now();
        assert!(!needs_refresh(now, now));
        assert!(!needs_refresh(
            now - SESSION_REFRESH_INTERVAL + TimeDelta::seconds(1),
            now
        ));
        assert!(needs_refresh(now - SESSION_REFRESH_INTERVAL, now));
    }

    #[test]
    fn https_cookie_is_secure_and_host_prefixed() {
        let cookie = manager("https://rss.example.com")
            .cookie("token".into(), Utc::now() + SESSION_IDLE_TIMEOUT);
        assert_eq!(cookie.name(), "__Host-wyrm_session");
        assert_eq!(cookie.value(), "token");
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[test]
    fn http_cookie_is_neither_secure_nor_host_prefixed() {
        let cookie = manager("http://localhost:5173")
            .cookie("token".into(), Utc::now() + SESSION_IDLE_TIMEOUT);
        assert_eq!(cookie.name(), "wyrm_session");
        assert_eq!(cookie.secure(), Some(false));
    }

    #[test]
    fn cookie_with_past_expiry_has_zero_max_age() {
        let cookie = manager("https://rss.example.com").cookie(String::new(), DateTime::UNIX_EPOCH);
        assert_eq!(cookie.max_age(), Some(CookieDuration::ZERO));
    }

    #[test]
    fn csrf_rejects_missing_or_wrong_header() {
        let sessions = manager("https://rss.example.com");

        let missing = TestRequest::default().to_http_request();
        assert!(is_csrf_failure(sessions.check_csrf(&missing)));

        let wrong = TestRequest::default()
            .insert_header(("x-wyrm-csrf", "0"))
            .to_http_request();
        assert!(is_csrf_failure(sessions.check_csrf(&wrong)));
    }

    #[test]
    fn csrf_accepts_header_with_matching_or_no_origin() {
        let sessions = manager("https://rss.example.com");

        let no_origin = TestRequest::default()
            .insert_header(("x-wyrm-csrf", "1"))
            .to_http_request();
        assert!(sessions.check_csrf(&no_origin).is_ok());

        let same_origin = TestRequest::default()
            .insert_header(("x-wyrm-csrf", "1"))
            .insert_header((header::ORIGIN, "https://rss.example.com"))
            .to_http_request();
        assert!(sessions.check_csrf(&same_origin).is_ok());
    }

    #[test]
    fn csrf_rejects_foreign_origin() {
        let sessions = manager("https://rss.example.com");
        let foreign = TestRequest::default()
            .insert_header(("x-wyrm-csrf", "1"))
            .insert_header((header::ORIGIN, "https://evil.example.com"))
            .to_http_request();
        assert!(is_csrf_failure(sessions.check_csrf(&foreign)));
    }
}
