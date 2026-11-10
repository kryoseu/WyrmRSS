use super::*;
use actix_web::{App, http::StatusCode, middleware::from_fn, test};
use openidconnect::{
    AuthUrl,
    EmptyAdditionalProviderMetadata,
    JsonWebKeySetUrl,
    ResponseTypes,
    core::{CoreJwsSigningAlgorithm, CoreResponseType, CoreSubjectIdentifierType},
};

const SESSION_COOKIE: &str = "__Host-wyrm_session";
const LOGIN_COOKIE: &str = "__Host-wyrm_oidc_login";

mod provider;

fn configured_auth() -> Auth {
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
            origin: "https://rss.example.com".into(),
            secure: true,
            allowed_subjects: vec!["owner".into()],
            login_key: Key::generate(),
            started_at: Instant::now(),
            sessions: Mutex::new(HashMap::new()),
        }),
    }
}

#[actix_web::test]
async fn defaults_preserve_open_access_and_api_key_only_access() {
    for key in [None, Some("secret".to_string())] {
        let config = WyrmStartupConfig {
            api_key: key.clone(),
            ..Default::default()
        };
        let auth = web::Data::new(Auth::new(&config).unwrap());
        assert!(auth.oidc.is_none());
        let app = test::init_service(
            App::new()
                .app_data(auth)
                .wrap(from_fn(middleware))
                .configure(configure)
                .route("/protected", web::get().to(HttpResponse::Ok)),
        )
        .await;
        let response = test::try_call_service(
            &app,
            test::TestRequest::get().uri("/protected").to_request(),
        )
        .await;
        if key.is_some() {
            assert_eq!(
                response.err().unwrap().as_response_error().status_code(),
                StatusCode::UNAUTHORIZED
            );
        } else {
            assert_eq!(response.unwrap().status(), StatusCode::OK);
        }
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/protected")
                .insert_header(("x-api-key", "secret"))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/auth/login")
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
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
async fn api_key_is_independent_of_oidc_and_sessions_require_csrf() {
    let auth = web::Data::new(configured_auth());
    auth.oidc
        .as_ref()
        .unwrap()
        .sessions
        .lock()
        .unwrap()
        .insert("session".into(), Instant::now() + SESSION_TTL);
    let app = test::init_service(
        App::new()
            .app_data(auth.clone())
            .wrap(from_fn(middleware))
            .configure(configure)
            .route("/protected", web::route().to(HttpResponse::Ok)),
    )
    .await;
    for method in [Method::GET, Method::POST, Method::PATCH, Method::DELETE] {
        let request = test::TestRequest::default()
            .method(method)
            .uri("/protected")
            .insert_header(("x-api-key", "api-secret"))
            .to_request();
        assert_eq!(
            test::call_service(&app, request).await.status(),
            StatusCode::OK
        );
    }
    for (token, csrf, origin, expected) in [
        ("invalid", None, None, StatusCode::UNAUTHORIZED),
        ("session", None, None, StatusCode::FORBIDDEN),
        (
            "session",
            Some("1"),
            Some("https://evil.example"),
            StatusCode::FORBIDDEN,
        ),
        (
            "session",
            Some("1"),
            Some("https://rss.example.com"),
            StatusCode::OK,
        ),
    ] {
        let mut request = test::TestRequest::post()
            .uri("/protected")
            .cookie(Cookie::new(SESSION_COOKIE, token));
        if let Some(csrf) = csrf {
            request = request.insert_header(("x-wyrm-csrf", csrf));
        }
        if let Some(origin) = origin {
            request = request.insert_header((header::ORIGIN, origin));
        }
        let response = test::try_call_service(&app, request.to_request()).await;
        let actual = match response {
            Ok(res) => res.status(),
            Err(err) => err.as_response_error().status_code(),
        };
        assert_eq!(actual, expected);
    }
    let request = test::TestRequest::get()
        .uri("/protected")
        .cookie(Cookie::new(SESSION_COOKIE, "session"))
        .to_request();
    assert_eq!(
        test::call_service(&app, request).await.status(),
        StatusCode::OK
    );
    let request = test::TestRequest::post()
        .uri("/api/v1/auth/logout")
        .insert_header(("x-wyrm-csrf", "1"))
        .cookie(Cookie::new(SESSION_COOKIE, "session"))
        .to_request();
    assert_eq!(
        test::call_service(&app, request).await.status(),
        StatusCode::NO_CONTENT
    );
    assert!(
        auth.oidc
            .as_ref()
            .unwrap()
            .sessions
            .lock()
            .unwrap()
            .is_empty()
    );
}

#[actix_web::test]
async fn oidc_only_is_protected_and_expired_sessions_are_rejected() {
    let mut auth = configured_auth();
    auth.api_key = None;
    auth.oidc
        .as_ref()
        .unwrap()
        .sessions
        .lock()
        .unwrap()
        .insert("expired".into(), Instant::now() - Duration::from_secs(1));
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(auth))
            .wrap(from_fn(middleware))
            .configure(configure)
            .route("/protected", web::get().to(HttpResponse::Ok)),
    )
    .await;
    for token in ["expired", "forged"] {
        let request = test::TestRequest::get()
            .uri("/protected")
            .cookie(Cookie::new(SESSION_COOKIE, token))
            .insert_header(("x-api-key", "anything"))
            .to_request();
        assert_eq!(
            test::try_call_service(&app, request)
                .await
                .err()
                .unwrap()
                .as_response_error()
                .status_code(),
            StatusCode::UNAUTHORIZED
        );
    }
    let config: serde_json::Value = test::call_and_read_body_json(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/auth/config")
            .to_request(),
    )
    .await;
    assert_eq!(
        config,
        serde_json::json!({ "oidc_enabled": true, "api_key_enabled": false, "oidc_session": false })
    );
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
        .seal_login(&PendingLogin {
            state: CsrfToken::new("state".into()),
            nonce: Nonce::new_random(),
            verifier: PkceCodeVerifier::new("verifier".into()),
            expires_at: 0,
            return_to: "/".into(),
        })
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
