use actix_web::web;
use api_api::{feeds, webhook as api_webhook};
use api_crud::{feeds as crud_feeds, webhook as crud_webhook};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/feeds")
            .route("", web::get().to(feeds::list))
            .route("", web::post().to(crud_feeds::create))
            .route("/poll", web::post().to(feeds::poll))
            .route("/{feed_id}", web::get().to(feeds::get))
            .route("/{feed_id}/icon", web::get().to(feeds::icon))
            .route("/{feed_id}", web::patch().to(crud_feeds::update))
            .route("/{feed_id}", web::delete().to(crud_feeds::delete))
            .route(
                "/{feed_id}/webhooks",
                web::get().to(api_webhook::list_for_feed),
            )
            .route(
                "/{feed_id}/webhooks/{webhook_id}",
                web::put().to(crud_webhook::attach),
            )
            .route(
                "/{feed_id}/webhooks/{webhook_id}",
                web::delete().to(crud_webhook::detach),
            ),
    );
}
