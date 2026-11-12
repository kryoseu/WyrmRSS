use crate::{Auth, api_key, session};
use actix_web::{
    Error,
    HttpMessage,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    http::Method,
    middleware::Next,
    web,
};

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
    let key_valid = api_key::valid(req.request(), auth);
    let check_session = (!public && !key_valid) || req.path() == "/api/v1/auth/config";
    let cookie = match (check_session, auth.session.as_ref()) {
        (true, Some(config)) => session::validate(req.request(), config).await?,
        _ => None,
    };
    if !public && !key_valid {
        if cookie.is_some() {
            if !matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
                session::check_csrf(req.request(), auth.session.as_ref().unwrap())?;
            }
        } else if auth.api_key_enabled() || auth.oidc_enabled() {
            return Err(ErrorUnauthorized("Authentication required"));
        }
    }
    if cookie.is_some() {
        req.extensions_mut().insert(session::AuthenticatedSession);
    }
    let mut response = next.call(req).await?;
    if let Some(cookie) = cookie {
        response.response_mut().add_cookie(&cookie)?;
    }
    Ok(response)
}

#[cfg(test)]
mod tests;
