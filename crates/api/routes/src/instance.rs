use actix_web::web;

pub fn public_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/instance", web::get().to(api_api::instance::get));
}
