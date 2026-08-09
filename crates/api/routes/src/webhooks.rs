use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/webhooks")
            .route("", web::get().to(api_api::webhook::list))
            .route("", web::post().to(api_crud::webhook::create))
            .route("/{webhook_id}", web::patch().to(api_crud::webhook::update))
            .route("/{webhook_id}", web::delete().to(api_crud::webhook::delete)),
    );
}
