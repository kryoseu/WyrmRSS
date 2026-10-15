use actix_web::web;
use api_api::{settings, shortcuts};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/settings")
            .route("", web::get().to(settings::get))
            .route("", web::put().to(api_crud::settings::update))
            .service(
                web::scope("/shortcuts")
                    .route("", web::get().to(shortcuts::list))
                    .route("/{action}", web::put().to(api_crud::shortcuts::update))
                    .route("/{action}", web::delete().to(api_crud::shortcuts::delete)),
            )
            .service(
                web::scope("/opml")
                    .route("/import", web::post().to(settings::import))
                    .route("/export", web::get().to(settings::export)),
            ),
    );
}
