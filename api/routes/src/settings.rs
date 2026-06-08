use actix_web::web;
use api_api::settings;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/settings").service(
            web::scope("/opml")
                .route("/import", web::post().to(settings::import))
                .route("/export", web::get().to(settings::export)),
        ),
    );
}
