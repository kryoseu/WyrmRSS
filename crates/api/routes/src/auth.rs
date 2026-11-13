use actix_web::{HttpResponse, web};
use api_api::auth;

/// Reachable without signing in: the sign-in flow, and logout so an expired session can clear its
/// cookie.
pub fn public_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/auth/status", web::get().to(auth::status))
        .route("/auth/oidc/login", web::get().to(auth::oidc_login))
        .route("/auth/oidc/callback", web::get().to(auth::oidc_callback))
        .route("/auth/logout", web::post().to(api_crud::auth::logout));
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").route(
        "/verify",
        web::get().to(|| async { HttpResponse::Ok().finish() }),
    ));
}
