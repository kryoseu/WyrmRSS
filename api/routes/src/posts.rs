use actix_web::web;
use api_api::{
    archive,
    posts::{self},
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/feeds/{feed_id}/posts").route("", web::get().to(posts::list_by_feed)))
        .service(
            web::scope("/posts")
                .route("", web::get().to(posts::list))
                .route("/favorites", web::get().to(posts::list_favorites))
                .service(
                    web::scope("/archive")
                        .route("", web::get().to(archive::list))
                        .route("/{post_id}", web::get().to(archive::get))
                        .route("/{post_id}", web::delete().to(posts::unarchive))
                        .route("/{post_id}", web::post().to(posts::archive)),
                )
                .route("/{post_id}", web::get().to(posts::get))
                .route("/{post_id}", web::patch().to(api_crud::posts::update)),
        );
}
