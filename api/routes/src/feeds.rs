use actix_web::web;
use api_api::feeds;
use api_crud::feeds as crud_feeds;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/feeds")
            .route("", web::get().to(feeds::list))
            .route("", web::post().to(crud_feeds::create))
            .route("/{feed_id}", web::get().to(feeds::get))
            .route("/{feed_id}", web::patch().to(crud_feeds::update))
            .route("/{feed_id}", web::delete().to(crud_feeds::delete)),
    );
}
