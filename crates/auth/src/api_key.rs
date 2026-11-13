use crate::Auth;
use actix_web::HttpRequest;
use sha2::{Digest, Sha256};

pub(crate) fn valid(req: &HttpRequest, auth: &Auth) -> bool {
    auth.api_key.as_deref().is_some_and(|key| {
        let provided = req
            .headers()
            .get("x-api-key")
            .and_then(|value| value.to_str().ok());
        provided.is_some_and(|provided| Sha256::digest(provided) == Sha256::digest(key))
    })
}
