use actix_web::web;
use api_api::{feeds, webhook as api_webhook};
use api_crud::{feeds as crud_feeds, webhook as crud_webhook};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/feeds")
            .route("", web::get().to(feeds::list))
            .route("", web::post().to(crud_feeds::create))
            .route("/poll", web::post().to(feeds::poll))
            .route("/{id}", web::get().to(feeds::get))
            .route("/{id}", web::patch().to(crud_feeds::update))
            .route("/{id}", web::delete().to(crud_feeds::delete))
            .route("/{id}/icon", web::get().to(feeds::icon))
            .route("/{id}/webhooks", web::get().to(api_webhook::list_for_feed))
            .route(
                "/{id}/webhooks/{webhook_id}",
                web::put().to(crud_webhook::attach),
            )
            .route(
                "/{id}/webhooks/{webhook_id}",
                web::delete().to(crud_webhook::detach),
            ),
    );
}
