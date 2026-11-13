use actix_web::{HttpRequest, HttpResponse, http::header, web::Data};
use api_utils::context::WyrmContext;
use serde::Serialize;
use wyrm_auth::{Auth, AuthMethod};
use wyrm_utils::result::WyrmResult;

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct AuthStatusResponse {
    pub methods: Vec<AuthMethod>,
    pub signed_in: bool,
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
