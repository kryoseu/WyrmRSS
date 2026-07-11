use actix_web::web;
use api_api::{
    archive,
    posts::{self},
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/posts")
            .route("", web::get().to(posts::list))
            .service(
                web::scope("/archive")
                    .route("", web::get().to(archive::list))
                    .route("/{id}", web::get().to(archive::get))
                    .route("/{id}", web::delete().to(posts::unarchive))
                    .route("/{id}", web::post().to(posts::archive)),
            )
            .route("/mark-read", web::post().to(posts::mark_as_read))
            .route("/{id}", web::get().to(posts::get))
            .route("/{id}", web::patch().to(api_crud::posts::update)),
    );
}
