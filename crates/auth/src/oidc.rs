//! OpenID Connect discovery, authorization, and callback verification.
use actix_web::{
    HttpRequest,
    cookie::{Cookie, CookieJar, Key, SameSite, time::Duration as CookieDuration},
};
use chrono::Utc;
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
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, time::Duration};
use url::Url;
use wyrm_utils::{
    config::WyrmStartupConfig,
    error::{AuthError, WyrmError},
    result::WyrmResult,
};

const LOGIN_TTL: Duration = Duration::from_secs(600);
const RETURN_PATH_LIMIT: usize = 1024;

/// OIDC client built from discovery; auth and token endpoints guaranteed.
type ProviderClient = CoreClient<
    EndpointSet,      // authorization
    EndpointNotSet,   // device authorization
    EndpointNotSet,   // introspection
    EndpointNotSet,   // revocation
    EndpointSet,      // token
    EndpointMaybeSet, // userinfo
>;

/// OIDC sign-in: verifies who the user is at the provider; never creates sessions.
pub struct Oidc {
    /// Client for discovery and token exchange; never follows redirects.
    http: reqwest::Client,
    /// Provider's issuer exactly as configured (not normalized): discovery compares it as a
    /// string.
    issuer: IssuerUrl,
    /// Client ID registered at the provider.
    client_id: String,
    /// `None` for a public client, which relies on PKCE alone.
    client_secret: Option<String>,
    /// Callback URL the provider sends the browser back to.
    redirect: RedirectUrl,
    /// Subjects (`sub` claims) allowed to sign in; `None` allows everyone.
    allowed_subjects: Option<Vec<String>>,
    /// Served over HTTPS: login cookie gets `Secure` and the `__Host-` prefix.
    secure: bool,
    /// Per-process key encrypting the login cookie; a restart voids pending logins.
    login_key: Key,
}

impl Oidc {
    pub(crate) fn new(config: &WyrmStartupConfig) -> WyrmResult<Option<Self>> {
        let enabled = config.oidc_issuer_url.is_some();

        if enabled {
            let public_url = config
                .public_url()?
                .ok_or_else(|| invalid_config("public url required"))?;

            validate_url(public_url.as_ref())?;

            let issuer_url = config
                .oidc_issuer_url
                .as_deref()
                .ok_or_else(|| invalid_config("issuer url required"))?;

            validate_url(issuer_url)?;

            let client_id = config
                .oidc_client_id
                .clone()
                .ok_or_else(|| invalid_config("client id required"))?;

            let client_secret = config.oidc_client_secret.clone();

            let allowed_subjects = config
                .oidc_allowed_subjects
                .as_deref()
                .map(parse_allowed_subjects)
                .transpose()?;

            let issuer = IssuerUrl::new(issuer_url.to_owned())
                .map_err(|_| invalid_config("invalid issuer url"))?;

            let callback_url = public_url
                .join("/api/v1/auth/oidc/callback")
                .map_err(|_| invalid_config("could not build callback url"))?;

            let redirect = RedirectUrl::from_url(callback_url);

            let login_key = Key::try_generate()
                .ok_or_else(|| invalid_config("could not generate login cookie key"))?;

            let oidc = Self {
                http: crate::http::client()?,
                issuer,
                client_id,
                client_secret,
                redirect,
                allowed_subjects,
                secure: public_url.scheme() == "https",
                login_key,
            };

            return Ok(Some(oidc));
        }
        Ok(None)
    }

    async fn client(&self) -> WyrmResult<ProviderClient> {
        let metadata = self.discover().await?;

        let token_url = metadata
            .token_endpoint()
            .cloned()
            .ok_or_else(|| provider_unavailable("identity provider has no token endpoint"))?;

        let client_id = ClientId::new(self.client_id.clone());
        let client_secret = self.client_secret.clone().map(ClientSecret::new);
        Ok(
            CoreClient::from_provider_metadata(metadata, client_id, client_secret)
                .set_token_uri(token_url)
                .set_redirect_uri(self.redirect.clone()),
        )
    }

    /// Builds an OIDC client from freshly discovered provider metadata.
    /// Starts a login: returns the provider URL to redirect to and the sealed login cookie.
    pub async fn begin_login(
        &self,
        return_to: Option<String>,
    ) -> WyrmResult<(Url, Cookie<'static>)> {
        let client = self.client().await?;

        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(challenge)
            .url();

        // Pending state lives only in the browser's encrypted cookie, never on the server.
        let pending = PendingLogin {
            state,
            nonce,
            verifier,
            expires_at: (Utc::now() + LOGIN_TTL).timestamp(),
            return_to: return_to
                .filter(|path| path.len() <= RETURN_PATH_LIMIT && local_path(path))
                .unwrap_or_else(|| "/".into()),
        };

        Ok((url, self.seal_login(&pending)))
    }

    /// Fetches the provider's metadata and signing keys, and checks its endpoints.
    async fn discover(&self) -> WyrmResult<CoreProviderMetadata> {
        let send = |request| crate::http::send(&self.http, request);
        let metadata = CoreProviderMetadata::discover_async(self.issuer.clone(), &send)
            .await
            .map_err(provider_unavailable)?;

        let endpoints = [
            Some(metadata.authorization_endpoint().url()),
            metadata.token_endpoint().map(|endpoint| endpoint.url()),
            Some(metadata.jwks_uri().url()),
        ];

        for url in endpoints.into_iter().flatten() {
            validate_url(url.as_str())
                .map_err(|_| provider_unavailable("insecure identity provider endpoint"))?;
        }

        Ok(metadata)
    }

    /// Encrypts the pending login into the cookie the callback reads back.
    fn seal_login(&self, pending: &PendingLogin) -> Cookie<'static> {
        let payload = serde_json::to_string(pending).expect("PendingLogin serializes to JSON");
        let mut jar = CookieJar::new();
        jar.private_mut(&self.login_key)
            .add(self.login_cookie(payload, LOGIN_TTL));
        jar.get(self.login_cookie_name())
            .expect("login cookie was just added")
            .clone()
    }

    /// Finishes a login: verifies the callback against the login cookie, returns the return_to
    /// path.
    pub async fn complete_login(
        &self,
        req: &HttpRequest,
        state: &str,
        code: &str,
    ) -> WyrmResult<String> {
        let cookie = req
            .cookie(self.login_cookie_name())
            .ok_or_else(|| login_failed("missing login cookie"))?;

        let pending = self
            .open_login(cookie, state)
            .ok_or_else(|| login_failed("invalid or expired login cookie"))?;

        // Discover on every login so rotated provider signing keys are picked up.
        let client = self.client().await?;

        let send = |request| crate::http::send(&self.http, request);

        let tokens = client
            .exchange_code(AuthorizationCode::new(code.to_owned()))
            .set_pkce_verifier(pending.verifier)
            .request_async(&send)
            .await
            .map_err(|error| login_failed(format!("code exchange: {error}")))?;

        let id_token = tokens
            .id_token()
            .ok_or_else(|| login_failed("no ID token in token response"))?;

        let claims = id_token
            .claims(&client.id_token_verifier(), &pending.nonce)
            .map_err(|error| login_failed(format!("ID token: {error}")))?;

        let subject = claims.subject().as_str();

        let allowed = self
            .allowed_subjects
            .as_ref()
            .is_none_or(|subjects| subjects.iter().any(|allowed| allowed == subject));

        if !allowed {
            return Err(login_failed(format!("subject {subject} not allowed")));
        }

        Ok(pending.return_to)
    }

    fn open_login(&self, cookie: Cookie<'static>, state: &str) -> Option<PendingLogin> {
        let jar = CookieJar::new();
        let cookie = jar.private(&self.login_key).decrypt(cookie)?;
        let pending: PendingLogin = serde_json::from_str(cookie.value()).ok()?;
        (pending.expires_at > Utc::now().timestamp()
            && pending.state == CsrfToken::new(state.to_owned()))
        .then_some(pending)
    }

    fn login_cookie(&self, value: String, ttl: Duration) -> Cookie<'static> {
        Cookie::build(self.login_cookie_name(), value)
            .path("/")
            .http_only(true)
            .secure(self.secure)
            .same_site(SameSite::Lax)
            .max_age(CookieDuration::seconds(ttl.as_secs() as i64))
            .finish()
    }

    fn login_cookie_name(&self) -> &'static str {
        if self.secure {
            "__Host-wyrm_oidc_login"
        } else {
            "wyrm_oidc_login"
        }
    }

    pub fn clear_login_cookie(&self) -> Cookie<'static> {
        self.login_cookie(String::new(), Duration::ZERO)
    }
}

#[derive(Deserialize, Serialize)]
struct PendingLogin {
    state: CsrfToken,
    nonce: Nonce,
    verifier: PkceCodeVerifier,
    expires_at: i64,
    return_to: String,
}

pub(crate) fn invalid_config(message: impl Display) -> WyrmError {
    AuthError::InvalidAuthConfig(format!("OIDC: {message}")).into()
}

pub(crate) fn provider_unavailable(message: impl Display) -> WyrmError {
    AuthError::AuthProviderUnavailable(format!("OIDC: {message}")).into()
}

fn login_failed(reason: impl Display) -> WyrmError {
    AuthError::LoginFailed(format!("OIDC: {reason}")).into()
}

/// Parses a comma-separated subject list; an empty list is a config error, not "allow all".
fn parse_allowed_subjects(value: &str) -> WyrmResult<Vec<String>> {
    let subjects: Vec<String> = value
        .split(',')
        .map(str::trim)
        .filter(|subject| !subject.is_empty())
        .map(String::from)
        .collect();

    if subjects.is_empty() {
        return Err(invalid_config("allowed subjects must not be empty"));
    }

    Ok(subjects)
}

fn validate_url(value: &str) -> WyrmResult<reqwest::Url> {
    let url = reqwest::Url::parse(value).map_err(|_| invalid_config("invalid URL"))?;
    if (url.scheme() != "https" && !(url.scheme() == "http" && is_loopback(&url)))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.query().is_some()
    {
        return Err(invalid_config("invalid url"));
    }
    Ok(url)
}

/// Loopback hosts, where plain HTTP is acceptable for local development.
fn is_loopback(url: &reqwest::Url) -> bool {
    matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
}

fn local_path(path: &str) -> bool {
    path.starts_with('/')
        && !path.starts_with("//")
        && !path.contains('\\')
        && !path.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> WyrmStartupConfig {
        WyrmStartupConfig {
            public_url: Some("https://rss.example.com".into()),
            oidc_issuer_url: Some("https://sso.example.com/realms/wyrm".into()),
            oidc_client_id: Some("wyrm".into()),
            ..Default::default()
        }
    }

    fn oidc() -> Oidc {
        Oidc::new(&config())
            .expect("valid config")
            .expect("OIDC enabled")
    }

    fn is_invalid_config(result: WyrmResult<Option<Oidc>>) -> bool {
        matches!(
            result,
            Err(WyrmError::Auth(AuthError::InvalidAuthConfig(_)))
        )
    }

    fn pending_login(expires_at: i64) -> PendingLogin {
        PendingLogin {
            state: CsrfToken::new("state".into()),
            nonce: Nonce::new("nonce".into()),
            verifier: PkceCodeVerifier::new("verifier".into()),
            expires_at,
            return_to: "/feeds".into(),
        }
    }

    fn in_ten_minutes() -> i64 {
        (Utc::now() + LOGIN_TTL).timestamp()
    }

    #[test]
    fn disabled_without_issuer() {
        let config = WyrmStartupConfig {
            oidc_issuer_url: None,
            ..config()
        };
        assert!(Oidc::new(&config).expect("valid config").is_none());
    }

    #[test]
    fn requires_public_url_and_client_id() {
        assert!(is_invalid_config(Oidc::new(&WyrmStartupConfig {
            public_url: None,
            ..config()
        })));
        assert!(is_invalid_config(Oidc::new(&WyrmStartupConfig {
            oidc_client_id: None,
            ..config()
        })));
    }

    #[test]
    fn plain_http_issuer_only_allowed_on_loopback() {
        assert!(is_invalid_config(Oidc::new(&WyrmStartupConfig {
            oidc_issuer_url: Some("http://sso.example.com".into()),
            ..config()
        })));
        assert!(
            Oidc::new(&WyrmStartupConfig {
                public_url: Some("http://localhost:5173".into()),
                oidc_issuer_url: Some("http://localhost:8080/default".into()),
                ..config()
            })
            .expect("valid config")
            .is_some()
        );
    }

    #[test]
    fn empty_allowed_subjects_is_a_config_error() {
        assert!(is_invalid_config(Oidc::new(&WyrmStartupConfig {
            oidc_allowed_subjects: Some(" , ".into()),
            ..config()
        })));
    }

    #[test]
    fn validate_url_rejects_credentials_query_and_fragment() {
        assert!(validate_url("https://sso.example.com/realms/wyrm").is_ok());
        assert!(validate_url("https://user:pass@sso.example.com").is_err());
        assert!(validate_url("https://sso.example.com?tenant=1").is_err());
        assert!(validate_url("https://sso.example.com#fragment").is_err());
        assert!(validate_url("ftp://sso.example.com").is_err());
    }

    #[test]
    fn local_path_rejects_external_redirects() {
        assert!(local_path("/feeds?id=1#top"));
        assert!(!local_path("https://evil.example.com"));
        assert!(!local_path("//evil.example.com"));
        assert!(!local_path("/\\evil.example.com"));
        assert!(!local_path("/feeds\n"));
    }

    #[test]
    fn login_cookie_round_trips_with_matching_state() {
        let oidc = oidc();
        let cookie = oidc.seal_login(&pending_login(in_ten_minutes()));
        assert_eq!(cookie.name(), "__Host-wyrm_oidc_login");
        assert_ne!(cookie.value(), "", "cookie should carry the sealed login");

        let pending = oidc
            .open_login(cookie, "state")
            .expect("valid login cookie");
        assert_eq!(pending.return_to, "/feeds");
    }

    #[test]
    fn login_cookie_rejects_wrong_state() {
        let oidc = oidc();
        let cookie = oidc.seal_login(&pending_login(in_ten_minutes()));
        assert!(oidc.open_login(cookie, "other-state").is_none());
    }

    #[test]
    fn login_cookie_rejects_expired_login() {
        let oidc = oidc();
        let cookie = oidc.seal_login(&pending_login(Utc::now().timestamp() - 1));
        assert!(oidc.open_login(cookie, "state").is_none());
    }

    #[test]
    fn login_cookie_rejects_tampering_and_other_keys() {
        let oidc = oidc();
        let cookie = oidc.seal_login(&pending_login(in_ten_minutes()));

        let tampered = Cookie::new(cookie.name().to_owned(), format!("{}x", cookie.value()));
        assert!(oidc.open_login(tampered, "state").is_none());

        // A restart generates a new key, voiding pending logins.
        let restarted = Oidc::new(&config())
            .expect("valid config")
            .expect("OIDC enabled");
        assert!(restarted.open_login(cookie, "state").is_none());
    }
}
