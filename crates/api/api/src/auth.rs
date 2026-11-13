use actix_web::{
    HttpRequest,
    HttpResponse,
    http::header,
    web::{self, Data, Query},
};
use api_utils::context::WyrmContext;
use serde::{Deserialize, Serialize};
use wyrm_auth::{Auth, AuthMethod, session::Authenticated};
use wyrm_utils::{error::AuthError, result::WyrmResult};

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct AuthStatusResponse {
    pub methods: Vec<AuthMethod>,
    pub signed_in: bool,
}

#[derive(Deserialize)]
pub struct OidcLoginQuery {
    return_to: Option<String>,
}

#[derive(Deserialize)]
pub struct OidcCallbackQuery {
    state: Option<String>,
    code: Option<String>,
    error: Option<String>,
}

pub async fn status(
    req: HttpRequest,
    ctx: Data<WyrmContext>,
    auth: Data<Auth>,
) -> WyrmResult<HttpResponse> {
    let signed_in = match auth.sessions() {
        Some(sessions) => sessions.authenticate(&ctx.db_pool, &req).await?.is_some(),
        None => false,
    };
    Ok(HttpResponse::Ok()
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .json(AuthStatusResponse {
            methods: auth.methods(),
            signed_in,
        }))
}

pub async fn oidc_login(
    query: web::Query<OidcLoginQuery>,
    auth: Data<Auth>,
) -> WyrmResult<HttpResponse> {
    let oidc = auth.oidc()?;
    let return_to = query.into_inner().return_to;

    let (location, cookie) = match oidc.begin_login(return_to).await {
        Ok((url, cookie)) => (url.to_string(), Some(cookie)),
        Err(error) => {
            tracing::warn!(%error, "OIDC login could not start");
            ("/?auth_error=oidc_unavailable".to_owned(), None)
        }
    };

    let mut response = HttpResponse::SeeOther();
    response
        .insert_header((header::LOCATION, location))
        .insert_header((header::CACHE_CONTROL, "no-store"));
    if let Some(cookie) = cookie {
        response.cookie(cookie);
    }
    Ok(response.finish())
}

/// A GET that creates a session: the provider redirects the browser here.
pub async fn oidc_callback(
    req: HttpRequest,
    query: Query<OidcCallbackQuery>,
    ctx: Data<WyrmContext>,
    auth: Data<Auth>,
) -> WyrmResult<HttpResponse> {
    let oidc = auth.oidc()?;

    let mut response = HttpResponse::SeeOther();
    response
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .insert_header((header::REFERRER_POLICY, "no-referrer"))
        .cookie(oidc.clear_login_cookie());

    match sign_in_with_oidc(&req, query.into_inner(), &auth, &ctx).await {
        Ok((return_to, authenticated)) => {
            response.insert_header((header::LOCATION, return_to));

            if let Some(cookie) = authenticated.cookie {
                response.cookie(cookie);
            }
        }
        Err(error) => {
            tracing::warn!(%error, "OIDC login failed");
            response.insert_header((header::LOCATION, "/?auth_error=oidc"));
        }
    }

    Ok(response.finish())
}

/// Verifies the callback with the provider, then starts a session.
async fn sign_in_with_oidc(
    req: &HttpRequest,
    query: OidcCallbackQuery,
    auth: &Auth,
    ctx: &WyrmContext,
) -> WyrmResult<(String, Authenticated)> {
    if let Some(error) = query.error {
        return Err(AuthError::LoginFailed(format!("OIDC: provider returned {error}")).into());
    }

    let (Some(state), Some(code)) = (query.state, query.code) else {
        return Err(AuthError::LoginFailed("OIDC: missing state or code".into()).into());
    };

    let return_to = auth.oidc()?.complete_login(req, &state, &code).await?;

    let sessions = auth
        .sessions()
        .ok_or_else(|| AuthError::AuthMethodDisabled("sessions".into()))?;

    let authenticated = sessions.create(&ctx.db_pool).await?;

    Ok((return_to, authenticated))
}
