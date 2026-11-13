//! Authentication for the shared instance; reader data is shared by all users.
mod api_key;
mod http;
pub mod middleware;
pub mod oidc;
pub mod session;

use crate::{oidc::Oidc, session::SessionManager};
use serde::Serialize;
use wyrm_utils::{
    config::WyrmStartupConfig,
    error::AuthError::{self},
    result::WyrmResult,
};

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
    oidc: Option<Oidc>,
    sessions: Option<SessionManager>,
}

impl Auth {
    pub fn new(config: &WyrmStartupConfig) -> WyrmResult<Self> {
        let oidc = Oidc::new(config)?;
        let sessions = config
            .public_url()?
            .map(|url| SessionManager::from_public_url(&url));

        Ok(Self {
            api_key: config.api_key.clone(),
            oidc,
            sessions,
        })
    }

    pub fn methods(&self) -> Vec<AuthMethod> {
        let mut methods = Vec::new();
        if self.api_key.is_some() {
            methods.push(AuthMethod::ApiKey);
        }
        if self.oidc.is_some() {
            methods.push(AuthMethod::Oidc);
        }
        methods
    }

    /// True when any sign-in method is configured; otherwise access is open.
    pub(crate) fn enabled(&self) -> bool {
        self.api_key.is_some() || self.oidc.is_some()
    }

    /// The OIDC method, or `AuthMethodDisabled` (404) when it isn't configured.
    pub fn oidc(&self) -> WyrmResult<&Oidc> {
        self.oidc
            .as_ref()
            .ok_or_else(|| AuthError::AuthMethodDisabled("oidc".into()).into())
    }

    pub fn sessions(&self) -> Option<&SessionManager> {
        self.sessions.as_ref()
    }
}
