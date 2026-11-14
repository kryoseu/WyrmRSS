//! Authentication for the shared instance; reader data is shared by all users.
mod api_key;
#[cfg(feature = "oidc")]
mod http;
pub mod middleware;
#[cfg(feature = "oidc")]
pub mod oidc;
pub mod session;

#[cfg(feature = "oidc")]
use crate::oidc::Oidc;
use crate::session::SessionManager;
use serde::Serialize;
#[cfg(feature = "oidc")]
use wyrm_utils::error::AuthError;
use wyrm_utils::{config::WyrmStartupConfig, result::WyrmResult};

/// Sign-in methods enabled on this instance, as reported to the frontend.
#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case", export)]
pub enum AuthMethod {
    ApiKey,
    Oidc,
}

pub struct Auth {
    api_key: Option<String>,
    #[cfg(feature = "oidc")]
    oidc: Option<Oidc>,
    sessions: Option<SessionManager>,
}

impl Auth {
    pub fn new(config: &WyrmStartupConfig) -> WyrmResult<Self> {
        let sessions = config
            .public_url()?
            .map(|url| SessionManager::from_public_url(&url));

        Ok(Self {
            api_key: config.api_key.clone(),
            #[cfg(feature = "oidc")]
            oidc: Oidc::new(config)?,
            sessions,
        })
    }

    pub fn methods(&self) -> Vec<AuthMethod> {
        let mut methods = Vec::new();
        if self.api_key.is_some() {
            methods.push(AuthMethod::ApiKey);
        }
        if self.oidc_enabled() {
            methods.push(AuthMethod::Oidc);
        }
        methods
    }

    /// True when any sign-in method is configured; otherwise access is open.
    pub(crate) fn enabled(&self) -> bool {
        self.api_key.is_some() || self.oidc_enabled()
    }

    /// True when OIDC is compiled in and configured.
    fn oidc_enabled(&self) -> bool {
        #[cfg(feature = "oidc")]
        let enabled = self.oidc.is_some();
        #[cfg(not(feature = "oidc"))]
        let enabled = false;
        enabled
    }

    /// The OIDC method, or `AuthMethodDisabled` (404) when it isn't configured.
    #[cfg(feature = "oidc")]
    pub fn oidc(&self) -> WyrmResult<&Oidc> {
        self.oidc
            .as_ref()
            .ok_or_else(|| AuthError::AuthMethodDisabled("oidc".into()).into())
    }

    pub fn sessions(&self) -> Option<&SessionManager> {
        self.sessions.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn api_key_config() -> WyrmStartupConfig {
        WyrmStartupConfig {
            api_key: Some("secret".into()),
            ..Default::default()
        }
    }

    #[test]
    fn no_methods_configured_leaves_access_open() {
        let auth = Auth::new(&WyrmStartupConfig::default()).expect("valid config");
        assert!(auth.methods().is_empty());
        assert!(!auth.enabled());
        assert!(auth.sessions().is_none());
    }

    #[test]
    fn api_key_alone_enables_auth() {
        let auth = Auth::new(&api_key_config()).expect("valid config");
        assert!(matches!(auth.methods().as_slice(), [AuthMethod::ApiKey]));
        assert!(auth.enabled());
    }

    #[cfg(feature = "oidc")]
    fn oidc_config() -> WyrmStartupConfig {
        WyrmStartupConfig {
            public_url: Some("https://rss.example.com".into()),
            oidc_issuer_url: Some("https://sso.example.com/realms/wyrm".into()),
            oidc_client_id: Some("wyrm".into()),
            ..Default::default()
        }
    }

    #[cfg(feature = "oidc")]
    #[test]
    fn oidc_alone_enables_auth() {
        let auth = Auth::new(&oidc_config()).expect("valid config");
        assert!(matches!(auth.methods().as_slice(), [AuthMethod::Oidc]));
        assert!(auth.enabled());
        assert!(auth.oidc().is_ok());
    }
}
