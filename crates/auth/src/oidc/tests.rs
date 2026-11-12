use super::*;
use crate::{
    middleware::middleware,
    tests::{LOGIN_COOKIE, configure},
};
use actix_web::{
    App,
    cookie::{SameSite, time::Duration as CookieDuration},
    http::StatusCode,
    middleware::from_fn,
    test,
};
use openidconnect::{
    AuthUrl,
    EmptyAdditionalProviderMetadata,
    JsonWebKeySetUrl,
    ResponseTypes,
    core::{CoreJwsSigningAlgorithm, CoreResponseType, CoreSubjectIdentifierType},
};
use std::collections::HashMap;

#[cfg(feature = "postgres")]
mod provider;

pub(crate) fn configured_auth() -> Auth {
    let issuer = IssuerUrl::new("https://id.example.com".into()).unwrap();
    let metadata = CoreProviderMetadata::new(
        issuer.clone(),
        AuthUrl::new("https://id.example.com/authorize".into()).unwrap(),
        JsonWebKeySetUrl::new("https://id.example.com/jwks".into()).unwrap(),
        vec![ResponseTypes::new(vec![CoreResponseType::Code])],
        vec![CoreSubjectIdentifierType::Public],
        vec![CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256],
        EmptyAdditionalProviderMetadata {},
    );
    let redirect = RedirectUrl::new("https://rss.example.com/api/v1/auth/callback".into()).unwrap();
    Auth {
        api_key: Some("api-secret".into()),
        session: Some(session::Config {
            origin: "https://rss.example.com".into(),
            secure: true,
        }),
        oidc: Some(Oidc {
            discovery: AsyncMutex::new(DiscoveryCache {
                client: Some(
                    CoreClient::from_provider_metadata(
                        metadata,
                        ClientId::new("wyrm".into()),
                        None,
                    )
                    .set_redirect_uri(redirect.clone()),
                ),
                refresh_at: Instant::now() + DISCOVERY_TTL,
            }),
            http: reqwest::Client::new(),
            issuer,
            client_id: "wyrm".into(),
            client_secret: None,
            redirect,
            allowed_subjects: Some(vec!["owner".into()]),
            login_key: Key::generate(),
            started_at: Instant::now(),
        }),
    }
}

#[actix_web::test]
async fn partial_configuration_fails_closed() {
    let config = WyrmStartupConfig {
        oidc_client_id: Some("wyrm".into()),
        ..Default::default()
    };
    assert!(Auth::new(&config).is_err());
}

#[actix_web::test]
async fn subject_filtering_is_optional_and_explicit_lists_are_validated() {
    let mut config = WyrmStartupConfig {
        oidc_issuer_url: Some("https://id.example.com".into()),
        oidc_client_id: Some("wyrm".into()),
        oidc_redirect_url: Some("https://rss.example.com/api/v1/auth/callback".into()),
        ..Default::default()
    };
    for (configured, expected) in [
        (None, None),
        (
            Some(" owner, guest, "),
            Some(vec!["owner".into(), "guest".into()]),
        ),
    ] {
        config.oidc_allowed_subjects = configured.map(String::from);
        assert_eq!(
            Auth::new(&config).unwrap().oidc.unwrap().allowed_subjects,
            expected
        );
    }
    for empty in ["", " ", ", ,"] {
        config.oidc_allowed_subjects = Some(empty.into());
        assert!(Auth::new(&config).is_err());
    }
}

#[actix_web::test]
async fn login_uses_pkce_bound_state_secure_cookies_and_local_redirects() {
    let auth = web::Data::new(configured_auth());
    let app = test::init_service(
        App::new()
            .app_data(auth.clone())
            .wrap(from_fn(middleware))
            .configure(configure),
    )
    .await;
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/auth/login?return_to=//evil.example")
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let url = reqwest::Url::parse(
        response
            .headers()
            .get(header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap(),
    )
    .unwrap();
    let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
    assert_eq!(params["response_type"], "code");
    assert_eq!(params["scope"], "openid");
    assert_eq!(params["code_challenge_method"], "S256");
    let login_cookie = response
        .response()
        .cookies()
        .find(|c| c.name() == LOGIN_COOKIE)
        .unwrap();
    assert_ne!(login_cookie.value(), params["state"]);
    assert_eq!(login_cookie.http_only(), Some(true));
    assert_eq!(login_cookie.secure(), Some(true));
    assert_eq!(login_cookie.path(), Some("/"));
    assert_eq!(login_cookie.same_site(), Some(SameSite::Lax));
    let oidc = auth.oidc.as_ref().unwrap();
    let pending = oidc
        .open_login(login_cookie.clone().into_owned(), &params["state"])
        .expect("the cookie contains the pending login");
    assert_eq!(pending.return_to, "/");
    assert_eq!(pending.nonce.secret(), &params["nonce"]);
    assert_eq!(
        PkceCodeChallenge::from_code_verifier_sha256(&pending.verifier).as_str(),
        params["code_challenge"]
    );
    assert!(
        oidc.open_login(login_cookie.clone().into_owned(), "different-state")
            .is_none()
    );
    let mut tampered = login_cookie.clone().into_owned();
    let mut value = tampered.value().as_bytes().to_vec();
    value[0] = if value[0] == b'A' { b'B' } else { b'A' };
    tampered.set_value(String::from_utf8(value).unwrap());
    assert!(oidc.open_login(tampered, &params["state"]).is_none());
    let callback_url = format!(
        "/api/v1/auth/callback?state={}&error=access_denied",
        params["state"]
    );
    let response = test::call_service(
        &app,
        test::TestRequest::get().uri(&callback_url).to_request(),
    )
    .await;
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/?auth_error=oidc"
    );
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&callback_url)
            .cookie(login_cookie.into_owned())
            .to_request(),
    )
    .await;
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/?auth_error=oidc"
    );
    let expired = response
        .response()
        .cookies()
        .find(|c| c.name() == LOGIN_COOKIE)
        .unwrap();
    assert_eq!(expired.max_age(), Some(CookieDuration::ZERO));
}

#[actix_web::test]
async fn expired_pending_login_is_rejected_even_if_the_browser_still_sends_it() {
    let auth = configured_auth();
    let oidc = auth.oidc.as_ref().unwrap();
    let cookie = oidc
        .seal_login(
            &PendingLogin {
                state: CsrfToken::new("state".into()),
                nonce: Nonce::new_random(),
                verifier: PkceCodeVerifier::new("verifier".into()),
                expires_at: 0,
                return_to: "/".into(),
            },
            auth.session.as_ref().unwrap(),
        )
        .unwrap();
    assert!(oidc.open_login(cookie, "state").is_none());
}

#[actix_web::test]
async fn abandoned_logins_do_not_block_new_attempts() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(configured_auth()))
            .wrap(from_fn(middleware))
            .configure(configure),
    )
    .await;
    // Fill the old global limit, then start one more login from a fresh browser.
    for _ in 0..1025 {
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/auth/login")
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert!(
            response
                .response()
                .cookies()
                .any(|c| c.name() == LOGIN_COOKIE)
        );
    }
}

#[actix_web::test]
async fn config_rejects_insecure_urls_and_external_return_paths() {
    for url in [
        "http://id.example.com",
        "https://user:secret@id.example.com",
        "https://id.example.com/#fragment",
    ] {
        assert!(validate_url(url).is_err());
    }
    assert!(validate_url("http://localhost:5173/api/v1/auth/callback").is_ok());
    for path in [
        "https://evil.example",
        "//evil.example",
        "/\\evil.example",
        "/\n/evil",
    ] {
        assert!(!local_path(path));
    }
    assert!(local_path("/subscribe?url=https%3A%2F%2Fexample.com#feed"));
}
