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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;
    use wyrm_utils::config::WyrmStartupConfig;

    fn auth_with_api_key(api_key: Option<&str>) -> Auth {
        Auth::new(&WyrmStartupConfig {
            api_key: api_key.map(String::from),
            ..Default::default()
        })
        .expect("valid config")
    }

    fn request_with_key(key: &str) -> HttpRequest {
        TestRequest::default()
            .insert_header(("x-api-key", key))
            .to_http_request()
    }

    #[test]
    fn accepts_matching_key() {
        assert!(valid(
            &request_with_key("secret"),
            &auth_with_api_key(Some("secret"))
        ));
    }

    #[test]
    fn rejects_wrong_or_missing_key() {
        let auth = auth_with_api_key(Some("secret"));
        assert!(!valid(&request_with_key("wrong"), &auth));
        assert!(!valid(&TestRequest::default().to_http_request(), &auth));
    }

    #[test]
    fn rejects_any_key_when_none_is_configured() {
        assert!(!valid(
            &request_with_key("secret"),
            &auth_with_api_key(None)
        ));
    }
}
