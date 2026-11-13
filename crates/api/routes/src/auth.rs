use actix_web::{HttpResponse, web};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route(
                "/verify",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            )
            .route("/config", web::get().to(wyrm_auth::config))
            .route("/login", web::get().to(wyrm_auth::oidc::login))
            .route("/callback", web::get().to(wyrm_auth::oidc::callback))
            .route("/logout", web::post().to(wyrm_auth::session::logout)),
    );
}
