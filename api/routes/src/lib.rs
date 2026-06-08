use actix_web::web;

mod feeds;
mod posts;
mod settings;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(posts::config)
            .configure(feeds::config)
            .configure(settings::config),
    );
}
