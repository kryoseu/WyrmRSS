use actix_web::{middleware::from_fn, web};

pub mod auth;
mod feeds;
mod folders;
mod host_credentials;
mod posts;
mod settings;
mod webhooks;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // Public routes are single resources: a scope here would claim its whole prefix.
            .configure(auth::public_config)
            .service(
                web::scope("")
                    .wrap(from_fn(wyrm_auth::middleware::middleware))
                    .configure(auth::config)
                    .configure(posts::config)
                    .configure(feeds::config)
                    .configure(webhooks::config)
                    .configure(settings::config)
                    .configure(folders::config)
                    .configure(host_credentials::config),
            ),
    );
}
