use crate::{Auth, oidc};
use actix_web::{
    Error, HttpRequest, HttpResponse,
    cookie::{Cookie, SameSite, time::Duration as CookieDuration},
    error::{ErrorForbidden, ErrorNotFound, ErrorServiceUnavailable},
    http::header,
    web,
};
use chrono::{DateTime, TimeDelta, Utc};
use database::{DatabasePool, models::session::Session};
use sha2::{Digest, Sha256};
use std::time::Duration;

pub(crate) const SESSION_IDLE_TIMEOUT: TimeDelta = TimeDelta::hours(12);

pub(crate) struct Config {
    pub origin: String,
    pub secure: bool,
}

pub(crate) struct AuthenticatedSession;

impl Config {
    pub(crate) fn cookie_name(&self) -> &'static str {
        if self.secure {
            "__Host-wyrm_session"
        } else {
            "wyrm_session"
        }
    }

    pub(crate) fn cookie(
        &self,
        name: &'static str,
        value: String,
        ttl: Duration,
    ) -> Cookie<'static> {
        Cookie::build(name, value)
            .path("/")
            .http_only(true)
            .secure(self.secure)
            .same_site(SameSite::Lax)
            .max_age(CookieDuration::seconds(ttl.as_secs() as i64))
            .finish()
    }

    fn session_cookie(&self, token: String, expires_at: DateTime<Utc>) -> Cookie<'static> {
        let remaining = (expires_at - Utc::now()).to_std().unwrap_or_default();
        self.cookie(self.cookie_name(), token, remaining)
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

fn database_pool(req: &HttpRequest) -> Result<&DatabasePool, Error> {
    req.app_data::<web::Data<DatabasePool>>()
        .map(web::Data::get_ref)
        .ok_or_else(|| ErrorServiceUnavailable("Session storage unavailable"))
}

pub(crate) async fn create(req: &HttpRequest, config: &Config) -> Result<Cookie<'static>, Error> {
    let token = generate_session_token();
    let now = Utc::now();
    let expires_at = now + SESSION_IDLE_TIMEOUT;
    let previous = req
        .cookie(config.cookie_name())
        .map(|cookie| token_hash(cookie.value()));
    Session::create(
        database_pool(req)?,
        Session {
            token_hash: token_hash(&token),
            expires_at,
            last_seen_at: now,
        },
        previous.as_deref(),
    )
    .await?;
    Ok(config.session_cookie(token, expires_at))
}

pub(crate) async fn validate(
    req: &HttpRequest,
    config: &Config,
) -> Result<Option<Cookie<'static>>, Error> {
    let Some(cookie) = req.cookie(config.cookie_name()) else {
        return Ok(None);
    };
    let now = Utc::now();
    let session = Session::refresh(
        database_pool(req)?,
        &token_hash(cookie.value()),
        now,
        now + SESSION_IDLE_TIMEOUT,
    )
    .await?;
    Ok(session.map(|session| config.session_cookie(cookie.value().to_owned(), session.expires_at)))
}

pub(crate) fn check_csrf(req: &HttpRequest, config: &Config) -> Result<(), Error> {
    if req
        .headers()
        .get("x-wyrm-csrf")
        .and_then(|v| v.to_str().ok())
        != Some("1")
        || req
            .headers()
            .get(header::ORIGIN)
            .is_some_and(|origin| origin.to_str().ok() != Some(&config.origin))
    {
        return Err(ErrorForbidden("Invalid request origin"));
    }
    Ok(())
}

pub async fn logout(req: HttpRequest, auth: web::Data<Auth>) -> Result<HttpResponse, Error> {
    let config = auth
        .session
        .as_ref()
        .ok_or_else(|| ErrorNotFound("OIDC disabled"))?;
    check_csrf(&req, config)?;
    if let Some(token) = req.cookie(config.cookie_name()) {
        Session::delete(database_pool(&req)?, &token_hash(token.value())).await?;
    }
    Ok(HttpResponse::NoContent()
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .cookie(config.cookie(config.cookie_name(), String::new(), Duration::ZERO))
        .cookie(oidc::clear_login_cookie(config))
        .finish())
}
