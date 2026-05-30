use actix_web::web;
use api_api::posts::{get, list, list_by_feed, list_favorites};
use api_crud::posts::update;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/feeds/{feed_id}/posts").route("", web::get().to(list_by_feed)))
        .service(
            web::scope("/posts")
                .route("", web::get().to(list))
                .route("/favorites", web::get().to(list_favorites))
                .route("/{post_id}", web::get().to(get))
                .route("/{post_id}", web::patch().to(update)),
        );
}
