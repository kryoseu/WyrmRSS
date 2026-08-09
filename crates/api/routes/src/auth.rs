use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").route(
        "/verify",
        web::get().to(|| async { actix_web::HttpResponse::Ok().finish() }),
    ));
}
