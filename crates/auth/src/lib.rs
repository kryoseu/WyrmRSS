//! Authentication for the shared instance; reader data is shared by all users.
mod api_key;
pub mod middleware;
pub mod oidc;
pub mod session;

use crate::session::SessionConfig;
use serde::Serialize;
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
    oidc: Option<oidc::Oidc>,
    session: Option<SessionConfig>,
}

impl Auth {
    pub fn new(config: &WyrmStartupConfig) -> WyrmResult<Self> {
        let oidc = oidc::Oidc::new(config)?;

        let session = oidc.as_ref().map(|oidc| {
            let redirect = oidc.redirect_url();
            SessionConfig {
                origin: redirect.origin().ascii_serialization(),
                secure: redirect.scheme() == "https",
            }
        });

        Ok(Self {
            api_key: config.api_key.clone(),
            oidc,
            session,
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
    pub(crate) fn required(&self) -> bool {
        self.api_key.is_some() || self.oidc.is_some()
    }
}
