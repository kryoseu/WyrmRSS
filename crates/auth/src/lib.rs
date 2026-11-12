//! Authentication for the shared instance; reader data is shared by all users.
mod api_key;
pub mod middleware;
pub mod oidc;
pub mod session;

use actix_web::{HttpMessage, HttpRequest, HttpResponse, http::header, web};
use wyrm_utils::{config::WyrmStartupConfig, result::WyrmResult};

pub struct Auth {
    api_key: Option<String>,
    oidc: Option<oidc::Oidc>,
    session: Option<session::Config>,
}

impl Auth {
    pub fn new(config: &WyrmStartupConfig) -> WyrmResult<Self> {
        let oidc = oidc::Oidc::new(config)?;
        let session = oidc.as_ref().map(|oidc| {
            let redirect = oidc.redirect_url();
            session::Config {
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

    pub fn api_key_enabled(&self) -> bool {
        self.api_key.is_some()
    }

    pub fn oidc_enabled(&self) -> bool {
        self.oidc.is_some()
    }
}

pub async fn config(req: HttpRequest, auth: web::Data<Auth>) -> HttpResponse {
    HttpResponse::Ok()
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .json(serde_json::json!({
            "oidc_enabled": auth.oidc_enabled(),
            "api_key_enabled": auth.api_key_enabled(),
            "oidc_session": req.extensions().get::<session::AuthenticatedSession>().is_some(),
        }))
}

#[cfg(test)]
mod tests;
