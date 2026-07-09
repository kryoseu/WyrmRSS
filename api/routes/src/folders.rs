use actix_web::web;
use api_api::folders;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/folders")
            .route("", web::get().to(folders::list))
            .route("", web::post().to(api_crud::folders::create))
            .route("/{id}", web::get().to(folders::get))
            .route("/{id}", web::patch().to(api_crud::folders::update))
            .route("/{id}", web::delete().to(api_crud::folders::delete)),
    );
}
