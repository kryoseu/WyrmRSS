//! OIDC protects the shared instance; it does not create separate reader accounts.
use actix_web::{
    Error,
    HttpRequest,
    HttpResponse,
    body::MessageBody,
    cookie::{Cookie, CookieJar, Key, SameSite, time::Duration as CookieDuration},
    dev::{ServiceRequest, ServiceResponse},
    error::{ErrorForbidden, ErrorServiceUnavailable, ErrorUnauthorized},
    http::{Method, header},
    middleware::Next,
    web,
};
use openidconnect::{
    AuthorizationCode,
    ClientId,
    ClientSecret,
    CsrfToken,
    EndpointMaybeSet,
    EndpointNotSet,
    EndpointSet,
    IssuerUrl,
    Nonce,
    PkceCodeChallenge,
    PkceCodeVerifier,
    RedirectUrl,
    TokenResponse,
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};
use tokio::sync::Mutex as AsyncMutex;
use wyrm_utils::{config::WyrmStartupConfig, error::WyrmError, result::WyrmResult};

const LOGIN_TTL: Duration = Duration::from_secs(600);
const SESSION_TTL: Duration = Duration::from_secs(12 * 60 * 60);
const SESSION_LIMIT: usize = 1024;
// Leave room for JSON escaping and cookie encryption within browser size limits.
const RETURN_PATH_LIMIT: usize = 1024;
const DISCOVERY_TTL: Duration = Duration::from_secs(300);
const DISCOVERY_RETRY: Duration = Duration::from_secs(5);
type Client = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

pub struct Auth {
    api_key: Option<String>,
    oidc: Option<Oidc>,
}

struct Oidc {
    discovery: AsyncMutex<DiscoveryCache>,
    http: reqwest::Client,
    issuer: IssuerUrl,
    client_id: String,
    client_secret: Option<String>,
    redirect: RedirectUrl,
    origin: String,
    secure: bool,
    allowed_subjects: Option<Vec<String>>,
    login_key: Key,
    started_at: Instant,
    sessions: Mutex<HashMap<String, Instant>>,
}

struct DiscoveryCache {
    client: Option<Client>,
    refresh_at: Instant,
}

#[derive(Deserialize, Serialize)]
struct PendingLogin {
    state: CsrfToken,
    nonce: Nonce,
    verifier: PkceCodeVerifier,
    expires_at: u64,
    return_to: String,
}

impl Oidc {
    async fn discover_client(&self) -> Result<Client, Error> {
        discover(
            &self.issuer,
            &self.client_id,
            self.client_secret.as_deref(),
            &self.redirect,
            &self.http,
        )
        .await
    }

    async fn authorization_client(&self) -> Result<Client, Error> {
        // One request refreshes discovery at a time. Back off after errors so a
        // burst of sign-in attempts cannot keep retrying an unavailable issuer.
        let mut cache = self.discovery.lock().await;
        if Instant::now() < cache.refresh_at {
            return cache
                .client
                .clone()
                .ok_or_else(|| ErrorServiceUnavailable("Identity provider discovery unavailable"));
        }
        let result = self.discover_client().await;
        cache.client = result.as_ref().ok().cloned();
        let delay = match &result {
            Ok(_) => DISCOVERY_TTL,
            Err(error) => {
                tracing::warn!(%error, "OIDC discovery failed");
                DISCOVERY_RETRY
            }
        };
        cache.refresh_at = Instant::now() + delay;
        result
    }

    fn session_cookie_name(&self) -> &'static str {
        if self.secure {
            "__Host-wyrm_session"
        } else {
            "wyrm_session"
        }
    }

    fn login_cookie_name(&self) -> &'static str {
        if self.secure {
            "__Host-wyrm_oidc_login"
        } else {
            "wyrm_oidc_login"
        }
    }

    fn seal_login(&self, pending: &PendingLogin) -> Result<Cookie<'static>, Error> {
        let payload = serde_json::to_string(pending)
            .map_err(|_| ErrorServiceUnavailable("Login unavailable"))?;
        let name = self.login_cookie_name();
        let mut jar = CookieJar::new();
        jar.private_mut(&self.login_key)
            .add(cookie(name, payload, LOGIN_TTL, self));
        Ok(jar.get(name).expect("login cookie was just added").clone())
    }

    fn open_login(&self, cookie: Cookie<'static>, state: &str) -> Option<PendingLogin> {
        let jar = CookieJar::new();
        let cookie = jar.private(&self.login_key).decrypt(cookie)?;
        let pending: PendingLogin = serde_json::from_str(cookie.value()).ok()?;
        (pending.expires_at > self.started_at.elapsed().as_secs()
            && pending.state == CsrfToken::new(state.to_owned()))
        .then_some(pending)
    }
}

fn invalid_config(message: &str) -> WyrmError {
    WyrmError::InvalidInput(format!("OIDC configuration: {message}"))
}

fn validate_url(value: &str) -> WyrmResult<reqwest::Url> {
    let url = reqwest::Url::parse(value).map_err(|_| invalid_config("invalid URL"))?;
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if (url.scheme() != "https" && !(url.scheme() == "http" && local))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.query().is_some()
    {
        return Err(invalid_config(
            "URLs must use HTTPS (HTTP is allowed on loopback only), without credentials, query, or fragment",
        ));
    }
    Ok(url)
}

impl Auth {
    pub fn new(config: &WyrmStartupConfig) -> WyrmResult<Self> {
        let options = [
            &config.oidc_issuer_url,
            &config.oidc_client_id,
            &config.oidc_client_secret,
            &config.oidc_redirect_url,
            &config.oidc_allowed_subjects,
        ];
        let oidc = if options.iter().any(|value| value.is_some()) {
            let required = |value: &Option<String>| {
                value
                    .clone()
                    .filter(|s| !s.trim().is_empty())
                    .ok_or_else(|| {
                        invalid_config("issuer URL, client ID, and redirect URL are required")
                    })
            };
            let issuer_value = required(&config.oidc_issuer_url)?;
            validate_url(&issuer_value)?;
            let redirect_value = required(&config.oidc_redirect_url)?;
            let redirect_url = validate_url(&redirect_value)?;
            if redirect_url.path() != "/api/v1/auth/callback" {
                return Err(invalid_config(
                    "redirect URL path must be /api/v1/auth/callback",
                ));
            }
            let allowed_subjects = config
                .oidc_allowed_subjects
                .as_deref()
                .map(|value| {
                    let subjects: Vec<String> = value
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(String::from)
                        .collect();
                    if subjects.is_empty() {
                        return Err(invalid_config(
                            "configured subject allowlist must not be empty",
                        ));
                    }
                    Ok(subjects)
                })
                .transpose()?;
            let http = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(15))
                .build()
                .map_err(|_| invalid_config("could not create HTTP client"))?;
            let issuer =
                IssuerUrl::new(issuer_value).map_err(|_| invalid_config("invalid issuer URL"))?;
            let redirect = RedirectUrl::new(redirect_value)
                .map_err(|_| invalid_config("invalid redirect URL"))?;
            let client_id = required(&config.oidc_client_id)?;
            let client_secret = config.oidc_client_secret.clone();
            Some(Oidc {
                discovery: AsyncMutex::new(DiscoveryCache {
                    client: None,
                    refresh_at: Instant::now(),
                }),
                http,
                issuer,
                client_id,
                client_secret,
                redirect,
                origin: redirect_url.origin().ascii_serialization(),
                secure: redirect_url.scheme() == "https",
                allowed_subjects,
                login_key: Key::try_generate()
                    .ok_or_else(|| invalid_config("could not generate a login cookie key"))?,
                started_at: Instant::now(),
                sessions: Mutex::new(HashMap::new()),
            })
        } else {
            None
        };
        Ok(Self {
            api_key: config.api_key.clone(),
            oidc,
        })
    }

    fn session_valid(&self, req: &HttpRequest) -> bool {
        let Some(oidc) = &self.oidc else { return false };
        let Some(cookie) = req.cookie(oidc.session_cookie_name()) else {
            return false;
        };
        let Ok(sessions) = oidc.sessions.lock() else {
            return false;
        };
        sessions
            .get(cookie.value())
            .is_some_and(|expiry| *expiry > Instant::now())
    }

    fn api_key_valid(&self, req: &HttpRequest) -> bool {
        self.api_key.as_deref().is_some_and(|key| {
            let provided = req
                .headers()
                .get("x-api-key")
                .and_then(|value| value.to_str().ok());
            provided.is_some_and(|provided| {
                CsrfToken::new(provided.to_owned()) == CsrfToken::new(key.to_owned())
            })
        })
    }
}

async fn discover(
    issuer: &IssuerUrl,
    id: &str,
    secret: Option<&str>,
    redirect: &RedirectUrl,
    http: &reqwest::Client,
) -> Result<Client, Error> {
    let metadata = CoreProviderMetadata::discover_async(issuer.clone(), http)
        .await
        .map_err(|_| ErrorServiceUnavailable("Identity provider discovery failed"))?;
    if metadata.token_endpoint().is_none() {
        return Err(ErrorServiceUnavailable(
            "Identity provider has no token endpoint",
        ));
    }
    // Discovered endpoints also need transport protection, especially for the client secret.
    for url in [
        metadata.authorization_endpoint().url(),
        metadata.token_endpoint().unwrap().url(),
        metadata.jwks_uri().url(),
    ] {
        validate_url(url.as_str())
            .map_err(|_| ErrorServiceUnavailable("Insecure identity provider endpoint"))?;
    }
    Ok(CoreClient::from_provider_metadata(
        metadata,
        ClientId::new(id.to_owned()),
        secret.map(|s| ClientSecret::new(s.to_owned())),
    )
    .set_redirect_uri(redirect.clone()))
}

fn check_csrf(req: &HttpRequest, oidc: &Oidc) -> Result<(), Error> {
    if req
        .headers()
        .get("x-wyrm-csrf")
        .and_then(|v| v.to_str().ok())
        != Some("1")
        || req
            .headers()
            .get(header::ORIGIN)
            .is_some_and(|origin| origin.to_str().ok() != Some(&oidc.origin))
    {
        return Err(ErrorForbidden("Invalid request origin"));
    }
    Ok(())
}

pub async fn middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let auth = req
        .app_data::<web::Data<Auth>>()
        .expect("auth data configured");
    let public = (req.method() == Method::GET
        && matches!(
            req.path(),
            "/api/v1/auth/config" | "/api/v1/auth/login" | "/api/v1/auth/callback"
        ))
        || (req.method() == Method::POST && req.path() == "/api/v1/auth/logout");
    if !public && !auth.api_key_valid(req.request()) {
        if auth.session_valid(req.request()) {
            if !matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
                check_csrf(req.request(), auth.oidc.as_ref().unwrap())?;
            }
        } else if auth.api_key.is_some() || auth.oidc.is_some() {
            return Err(ErrorUnauthorized("Authentication required"));
        }
    }
    next.call(req).await
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/v1/auth/config", web::get().to(auth_config))
        .route("/api/v1/auth/login", web::get().to(login))
        .route("/api/v1/auth/callback", web::get().to(callback))
        .route("/api/v1/auth/logout", web::post().to(logout));
}

async fn auth_config(req: HttpRequest, auth: web::Data<Auth>) -> HttpResponse {
    HttpResponse::Ok()
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .json(serde_json::json!({
            "oidc_enabled": auth.oidc.is_some(), "api_key_enabled": auth.api_key.is_some(),
            "oidc_session": auth.session_valid(&req),
        }))
}

fn cookie(name: &'static str, value: String, ttl: Duration, oidc: &Oidc) -> Cookie<'static> {
    Cookie::build(name, value)
        .path("/")
        .http_only(true)
        .secure(oidc.secure)
        .same_site(SameSite::Lax)
        .max_age(CookieDuration::seconds(ttl.as_secs() as i64))
        .finish()
}

fn local_path(path: &str) -> bool {
    path.starts_with('/')
        && !path.starts_with("//")
        && !path.contains('\\')
        && !path.chars().any(char::is_control)
}

#[derive(Deserialize)]
struct LoginQuery {
    return_to: Option<String>,
}

async fn login(
    query: web::Query<LoginQuery>,
    auth: web::Data<Auth>,
) -> Result<HttpResponse, Error> {
    let oidc = auth
        .oidc
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorNotFound("OIDC disabled"))?;
    let Ok(client) = oidc.authorization_client().await else {
        return Ok(HttpResponse::SeeOther()
            .insert_header((header::LOCATION, "/?auth_error=oidc_unavailable"))
            .insert_header((header::CACHE_CONTROL, "no-store"))
            .finish());
    };
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let (url, state, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .set_pkce_challenge(challenge)
        .url();
    // An abandoned login occupies only its browser's cookie. The server keeps
    // no pending entries that unauthenticated callers could fill up.
    let pending = PendingLogin {
        state,
        nonce,
        verifier,
        expires_at: oidc.started_at.elapsed().as_secs() + LOGIN_TTL.as_secs(),
        return_to: query
            .into_inner()
            .return_to
            .filter(|path| path.len() <= RETURN_PATH_LIMIT && local_path(path))
            .unwrap_or_else(|| "/".into()),
    };
    Ok(HttpResponse::SeeOther()
        .insert_header((header::LOCATION, url.as_str()))
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .cookie(oidc.seal_login(&pending)?)
        .finish())
}

#[derive(Deserialize)]
struct CallbackQuery {
    state: Option<String>,
    code: Option<String>,
    error: Option<String>,
}

async fn callback(
    req: HttpRequest,
    query: web::Query<CallbackQuery>,
    auth: web::Data<Auth>,
) -> Result<HttpResponse, Error> {
    let oidc = auth
        .oidc
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorNotFound("OIDC disabled"))?;
    let result = complete_login(&req, &query, oidc).await;
    let mut response = HttpResponse::SeeOther();
    response
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .insert_header((header::REFERRER_POLICY, "no-referrer"))
        .cookie(cookie(
            oidc.login_cookie_name(),
            String::new(),
            Duration::ZERO,
            oidc,
        ));
    match result {
        Ok((token, return_to)) => {
            // Rotate any existing session on successful authentication.
            if let Some(old) = req.cookie(oidc.session_cookie_name()) {
                oidc.sessions
                    .lock()
                    .map_err(|_| ErrorServiceUnavailable("Login unavailable"))?
                    .remove(old.value());
            }
            response
                .insert_header((header::LOCATION, return_to))
                .cookie(cookie(oidc.session_cookie_name(), token, SESSION_TTL, oidc));
        }
        Err(_) => {
            response.insert_header((header::LOCATION, "/?auth_error=oidc"));
        }
    }
    Ok(response.finish())
}

async fn complete_login(
    req: &HttpRequest,
    query: &CallbackQuery,
    oidc: &Oidc,
) -> Result<(String, String), Error> {
    let denied = || ErrorUnauthorized("OIDC login failed");
    let state = query.state.as_ref().ok_or_else(denied)?;
    let cookie = req.cookie(oidc.login_cookie_name()).ok_or_else(denied)?;
    let pending = oidc
        .open_login(cookie.into_owned(), state)
        .ok_or_else(denied)?;
    if query.error.is_some() {
        return Err(denied());
    }
    let code = query
        .code
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(denied)?;
    // Refresh discovery/JWKS for each completed login to support provider key rotation.
    let client = oidc.discover_client().await?;
    // The provider enforces single-use authorization codes. The callback also
    // clears the browser's login cookie when it returns.
    let tokens = client
        .exchange_code(AuthorizationCode::new(code))
        .map_err(|_| denied())?
        .set_pkce_verifier(pending.verifier)
        .request_async(&oidc.http)
        .await
        .map_err(|_| denied())?;
    let id_token = tokens.id_token().ok_or_else(denied)?;
    let claims = id_token
        .claims(&client.id_token_verifier(), &pending.nonce)
        .map_err(|_| denied())?;
    if oidc.allowed_subjects.as_ref().is_some_and(|subjects| {
        !subjects
            .iter()
            .any(|subject| subject == claims.subject().as_str())
    }) {
        return Err(denied());
    }
    let token = CsrfToken::new_random().secret().clone();
    let mut sessions = oidc.sessions.lock().map_err(|_| denied())?;
    sessions.retain(|_, expires| *expires > Instant::now());
    if sessions.len() >= SESSION_LIMIT {
        return Err(ErrorServiceUnavailable("Too many sessions"));
    }
    sessions.insert(token.clone(), Instant::now() + SESSION_TTL);
    Ok((token, pending.return_to))
}

async fn logout(req: HttpRequest, auth: web::Data<Auth>) -> Result<HttpResponse, Error> {
    let oidc = auth
        .oidc
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorNotFound("OIDC disabled"))?;
    check_csrf(&req, oidc)?;
    if let Some(token) = req.cookie(oidc.session_cookie_name()) {
        oidc.sessions
            .lock()
            .map_err(|_| ErrorServiceUnavailable("Logout unavailable"))?
            .remove(token.value());
    }
    Ok(HttpResponse::NoContent()
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .cookie(cookie(
            oidc.session_cookie_name(),
            String::new(),
            Duration::ZERO,
            oidc,
        ))
        .cookie(cookie(
            oidc.login_cookie_name(),
            String::new(),
            Duration::ZERO,
            oidc,
        ))
        .finish())
}

#[cfg(test)]
mod tests;
