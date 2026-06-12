use actix_web::web;
use actix_web_httpauth::extractors::bearer::BearerAuth;

pub async fn api_key_middleware(
    req: actix_web::dev::ServiceRequest,
    credentials: BearerAuth,
) -> Result<actix_web::dev::ServiceRequest, (actix_web::Error, actix_web::dev::ServiceRequest)> {
    let expected = req
        .app_data::<web::Data<Option<String>>>()
        .and_then(|k| k.as_deref());

    match expected {
        None => Ok(req),                                    // no key configured - pass
        Some(key) if credentials.token() == key => Ok(req), // key matches - pass
        _ => Err((actix_web::error::ErrorUnauthorized("Invalid api key"), req)),
    }
}
