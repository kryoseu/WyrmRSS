use actix_web::{HttpMessage, HttpRequest, HttpResponse, http::header, web};
use serde::Serialize;
use wyrm_auth::{Auth, AuthMethod, session};
use wyrm_utils::result::WyrmResult;

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct AuthStatusResponse {
    pub methods: Vec<AuthMethod>,
    pub signed_in: bool,
}

pub async fn status(req: HttpRequest, auth: web::Data<Auth>) -> WyrmResult<HttpResponse> {
    Ok(HttpResponse::Ok()
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .json(AuthStatusResponse {
            methods: auth.methods(),
            signed_in: req
                .extensions()
                .get::<session::AuthenticatedSession>()
                .is_some(),
        }))
}
