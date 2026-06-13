use actix_web::{Error, body::MessageBody, dev::ServiceResponse, middleware::Next, web};

pub async fn api_key_middleware(
    req: actix_web::dev::ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let api_key_from_conf = req
        .app_data::<web::Data<Option<String>>>()
        .and_then(|k| k.as_deref());

    if let Some(expected_key) = api_key_from_conf {
        let provided = req.headers().get("x-api-key").and_then(|v| v.to_str().ok());

        if provided != Some(expected_key) {
            return Err(actix_web::error::ErrorUnauthorized("Invalid api key"));
        }
    }

    // None = auth disabled, pass through
    next.call(req).await
}
