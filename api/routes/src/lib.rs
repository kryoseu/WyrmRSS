use actix_web::web;

mod auth;
mod feeds;
mod folders;
mod posts;
mod settings;
mod webhooks;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(auth::config)
            .configure(posts::config)
            .configure(feeds::config)
            .configure(webhooks::config)
            .configure(settings::config)
            .configure(folders::config),
    );
}
