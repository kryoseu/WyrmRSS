use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;
use wyrm_utils::middleware::api_key_middleware;

mod auth;
mod feeds;
mod posts;
mod settings;

pub fn config(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(api_key_middleware);
    cfg.service(
        web::scope("/api/v1")
            .wrap(auth)
            .configure(auth::config)
            .configure(posts::config)
            .configure(feeds::config)
            .configure(settings::config),
    );
}
