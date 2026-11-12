//! OpenID Connect discovery, authorization, and callback verification.
use crate::{Auth, session};
use actix_web::{
    Error,
    HttpRequest,
    HttpResponse,
    cookie::{Cookie, CookieJar, Key},
    error::{ErrorServiceUnavailable, ErrorUnauthorized},
    http::header,
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
use std::time::{Duration, Instant};
use tokio::sync::Mutex as AsyncMutex;
use wyrm_utils::{config::WyrmStartupConfig, error::WyrmError, result::WyrmResult};

const LOGIN_TTL: Duration = Duration::from_secs(600);
// Leave room for JSON escaping and cookie encryption within browser size limits.
const RETURN_PATH_LIMIT: usize = 1024;
const DISCOVERY_TTL: Duration = Duration::from_secs(300);
const DISCOVERY_RETRY: Duration = Duration::from_secs(5);
const OIDC_HTTP_TIMEOUT: Duration = Duration::from_secs(15);
type Client = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

pub(crate) struct Oidc {
    discovery: AsyncMutex<DiscoveryCache>,
    http: reqwest::Client,
    issuer: IssuerUrl,
    client_id: String,
    client_secret: Option<String>,
    redirect: RedirectUrl,
    allowed_subjects: Option<Vec<String>>,
    login_key: Key,
    started_at: Instant,
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
    pub(crate) fn new(config: &WyrmStartupConfig) -> WyrmResult<Option<Self>> {
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
                .timeout(OIDC_HTTP_TIMEOUT)
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
                allowed_subjects,
                login_key: Key::try_generate()
                    .ok_or_else(|| invalid_config("could not generate a login cookie key"))?,
                started_at: Instant::now(),
            })
        } else {
            None
        };
        Ok(oidc)
    }

    pub(crate) fn redirect_url(&self) -> &reqwest::Url {
        self.redirect.url()
    }

    async fn discover_client(&self) -> WyrmResult<Client> {
        let metadata = CoreProviderMetadata::discover_async(self.issuer.clone(), &self.http)
            .await
            .map_err(|error| WyrmError::AuthProviderUnavailable(error.to_string()))?;
        if metadata.token_endpoint().is_none() {
            return Err(WyrmError::AuthProviderUnavailable(
                "identity provider has no token endpoint".into(),
            ));
        }
        // Discovered endpoints also need transport protection, especially for the client secret.
        for url in [
            metadata.authorization_endpoint().url(),
            metadata.token_endpoint().unwrap().url(),
            metadata.jwks_uri().url(),
        ] {
            validate_url(url.as_str()).map_err(|_| {
                WyrmError::AuthProviderUnavailable("insecure identity provider endpoint".into())
            })?;
        }
        Ok(CoreClient::from_provider_metadata(
            metadata,
            ClientId::new(self.client_id.clone()),
            self.client_secret.clone().map(ClientSecret::new),
        )
        .set_redirect_uri(self.redirect.clone()))
    }

    async fn authorization_client(&self) -> WyrmResult<Client> {
        // One request refreshes discovery at a time. Back off after errors so a
        // burst of sign-in attempts cannot keep retrying an unavailable issuer.
        let mut cache = self.discovery.lock().await;
        if Instant::now() < cache.refresh_at {
            return cache.client.clone().ok_or_else(|| {
                WyrmError::AuthProviderUnavailable("identity provider discovery unavailable".into())
            });
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

    fn seal_login(
        &self,
        pending: &PendingLogin,
        config: &session::Config,
    ) -> Result<Cookie<'static>, Error> {
        let payload = serde_json::to_string(pending)
            .map_err(|_| ErrorServiceUnavailable("Login unavailable"))?;
        let name = login_cookie_name(config);
        let mut jar = CookieJar::new();
        jar.private_mut(&self.login_key)
            .add(config.cookie(name, payload, LOGIN_TTL));
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
    WyrmError::InvalidAuthConfig(message.to_owned())
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

fn login_cookie_name(config: &session::Config) -> &'static str {
    if config.secure {
        "__Host-wyrm_oidc_login"
    } else {
        "wyrm_oidc_login"
    }
}

pub(crate) fn clear_login_cookie(config: &session::Config) -> Cookie<'static> {
    config.cookie(login_cookie_name(config), String::new(), Duration::ZERO)
}

fn local_path(path: &str) -> bool {
    path.starts_with('/')
        && !path.starts_with("//")
        && !path.contains('\\')
        && !path.chars().any(char::is_control)
}

#[derive(Deserialize)]
pub struct LoginQuery {
    return_to: Option<String>,
}

pub async fn login(
    query: web::Query<LoginQuery>,
    auth: web::Data<Auth>,
) -> Result<HttpResponse, Error> {
    let oidc = auth
        .oidc
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorNotFound("OIDC disabled"))?;
    let config = auth.session.as_ref().expect("OIDC session configuration");
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
        .cookie(oidc.seal_login(&pending, config)?)
        .finish())
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    state: Option<String>,
    code: Option<String>,
    error: Option<String>,
}

pub async fn callback(
    req: HttpRequest,
    query: web::Query<CallbackQuery>,
    auth: web::Data<Auth>,
) -> Result<HttpResponse, Error> {
    let oidc = auth
        .oidc
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorNotFound("OIDC disabled"))?;
    let config = auth.session.as_ref().expect("OIDC session configuration");
    let result = complete_login(&req, &query, oidc, config).await;
    let mut response = HttpResponse::SeeOther();
    response
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .insert_header((header::REFERRER_POLICY, "no-referrer"))
        .cookie(clear_login_cookie(config));
    match result {
        Ok((cookie, return_to)) => {
            response
                .insert_header((header::LOCATION, return_to))
                .cookie(cookie);
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
    config: &session::Config,
) -> Result<(Cookie<'static>, String), Error> {
    let denied = || ErrorUnauthorized("OIDC login failed");
    let state = query.state.as_ref().ok_or_else(denied)?;
    let cookie = req.cookie(login_cookie_name(config)).ok_or_else(denied)?;
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
    let cookie = session::create(req, config).await?;
    Ok((cookie, pending.return_to))
}

#[cfg(test)]
pub(crate) mod tests;
