use actix_web::web;
use api_api::host_credentials;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/host-credential")
            .route("", web::get().to(host_credentials::list))
            .route("", web::post().to(api_crud::host_credentials::create))
            .route(
                "/{host_credential_id}",
                web::patch().to(api_crud::host_credentials::update),
            )
            .route(
                "/{host_credential_id}",
                web::delete().to(api_crud::host_credentials::delete),
            ),
    );
}
