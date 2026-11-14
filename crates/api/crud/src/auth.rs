use actix_web::{HttpRequest, HttpResponse, http::header, web::Data};
use api_utils::context::WyrmContext;
use wyrm_auth::Auth;
use wyrm_utils::{error::AuthError, result::WyrmResult};

pub async fn logout(
    req: HttpRequest,
    ctx: Data<WyrmContext>,
    auth: Data<Auth>,
) -> WyrmResult<HttpResponse> {
    let sessions = auth
        .sessions()
        .ok_or(AuthError::AuthMethodDisabled("sessions".into()))?;

    sessions.check_csrf(&req)?;

    let cookie = sessions.revoke(&ctx.db_pool, &req).await?;

    Ok(HttpResponse::NoContent()
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .cookie(cookie)
        .finish())
}
