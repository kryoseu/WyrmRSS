use crate::{Auth, api_key};
use actix_web::{
    Error,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    http::Method,
    middleware::Next,
    web,
};
use api_utils::context::WyrmContext;

pub async fn middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let ctx = req
        .app_data::<web::Data<WyrmContext>>()
        .expect("context configured");

    let auth = req
        .app_data::<web::Data<Auth>>()
        .expect("auth data configured");

    // No auth configured or a valid API key: nothing to check.
    if !auth.enabled() || api_key::valid(req.request(), auth) {
        return next.call(req).await;
    }

    // Otherwise the caller must have a valid session.
    let Some(sessions) = auth.sessions() else {
        return Err(ErrorUnauthorized("Authentication required"));
    };

    let Some(authenticated) = sessions.authenticate(&ctx.db_pool, req.request()).await? else {
        return Err(ErrorUnauthorized("Authentication required"));
    };

    if !safe_method(req.method()) {
        sessions.check_csrf(req.request())?;
    }

    let mut response = next.call(req).await?;
    if let Some(cookie) = authenticated.cookie {
        response.response_mut().add_cookie(&cookie)?;
    }
    Ok(response)
}

/// Read-only HTTP methods. They skip CSRF since they're read-only and don't alter server state.
fn safe_method(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}
