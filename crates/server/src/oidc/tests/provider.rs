use super::*;
use actix_web::HttpServer;
use chrono::{TimeDelta, Utc};
use openidconnect::{
    Audience,
    EmptyAdditionalClaims,
    JsonWebKeyId,
    PrivateSigningKey,
    StandardClaims,
    SubjectIdentifier,
    core::{CoreEdDsaPrivateSigningKey, CoreIdToken, CoreIdTokenClaims},
};
use std::{
    net::TcpListener,
    sync::atomic::{AtomicBool, Ordering},
};

// Test-only signing key for the loopback issuer.
const SIGNING_KEY: &str = "-----BEGIN PRIVATE KEY-----\n\
MC4CAQAwBQYDK2VwBCIEIJk6dXL6PuVs0coav8/ubciFD9/FISDzn4e20Fc/ECoP\n\
-----END PRIVATE KEY-----\n";
const REDIRECT_URI: &str = "http://localhost:5173/api/v1/auth/callback";

struct Exchange {
    challenge: String,
    nonce: String,
    subject: &'static str,
}

struct Provider {
    issuer: String,
    available: AtomicBool,
    exchange: Mutex<Option<Exchange>>,
    key: CoreEdDsaPrivateSigningKey,
}

async fn metadata(provider: web::Data<Provider>) -> HttpResponse {
    if !provider.available.load(Ordering::Relaxed) {
        return HttpResponse::ServiceUnavailable().finish();
    }
    HttpResponse::Ok().json(serde_json::json!({
        "issuer": provider.issuer,
        "authorization_endpoint": format!("{}/authorize", provider.issuer),
        "token_endpoint": format!("{}/token", provider.issuer),
        "jwks_uri": format!("{}/jwks", provider.issuer),
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["EdDSA"],
    }))
}

async fn keys(provider: web::Data<Provider>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "keys": [provider.key.as_verification_key()],
    }))
}

async fn token(
    provider: web::Data<Provider>,
    form: web::Form<HashMap<String, String>>,
) -> HttpResponse {
    let Some(exchange) = provider.exchange.lock().unwrap().take() else {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "invalid_grant"}));
    };
    let challenge = form.get("code_verifier").map(|verifier| {
        PkceCodeChallenge::from_code_verifier_sha256(&PkceCodeVerifier::new(verifier.clone()))
    });
    if challenge.as_ref().map(PkceCodeChallenge::as_str) != Some(&exchange.challenge)
        || form.get("grant_type").map(String::as_str) != Some("authorization_code")
        || form.get("code").map(String::as_str) != Some("test-code")
        || form.get("client_id").map(String::as_str) != Some("wyrm")
        || form.get("redirect_uri").map(String::as_str) != Some(REDIRECT_URI)
    {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "invalid_grant"}));
    }

    let now = Utc::now();
    let claims = CoreIdTokenClaims::new(
        IssuerUrl::new(provider.issuer.clone()).unwrap(),
        vec![Audience::new("wyrm".into())],
        now + TimeDelta::minutes(5),
        now,
        StandardClaims::new(SubjectIdentifier::new(exchange.subject.into())),
        EmptyAdditionalClaims {},
    )
    .set_nonce(Some(Nonce::new(exchange.nonce)));
    let id_token = CoreIdToken::new(
        claims,
        &provider.key,
        CoreJwsSigningAlgorithm::EdDsa,
        None,
        None,
    )
    .expect("sign test ID token");
    HttpResponse::Ok().json(serde_json::json!({
        "id_token": id_token.to_string(), "access_token": "test-access-token", "token_type": "Bearer",
    }))
}

#[actix_web::test]
async fn api_keys_survive_discovery_failure_and_signed_login_recovers() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let provider = web::Data::new(Provider {
        issuer: format!("http://{}", listener.local_addr().unwrap()),
        available: AtomicBool::new(false),
        exchange: Mutex::new(None),
        key: CoreEdDsaPrivateSigningKey::from_ed25519_pem(
            SIGNING_KEY,
            Some(JsonWebKeyId::new("test-key".into())),
        )
        .unwrap(),
    });
    let server = HttpServer::new({
        let provider = provider.clone();
        move || {
            App::new()
                .app_data(provider.clone())
                .route("/.well-known/openid-configuration", web::get().to(metadata))
                .route("/jwks", web::get().to(keys))
                .route("/token", web::post().to(token))
        }
    })
    .workers(1)
    .listen(listener)
    .unwrap()
    .run();
    let handle = server.handle();
    let task = actix_web::rt::spawn(server);

    let auth = web::Data::new(
        Auth::new(&WyrmStartupConfig {
            api_key: Some("test-api-key".into()),
            oidc_issuer_url: Some(provider.issuer.clone()),
            oidc_client_id: Some("wyrm".into()),
            oidc_redirect_url: Some(REDIRECT_URI.into()),
            oidc_allowed_subjects: Some("owner".into()),
            ..WyrmStartupConfig::default()
        })
        .expect("an unavailable issuer must not prevent startup"),
    );
    let app = test::init_service(
        App::new()
            .app_data(auth.clone())
            .wrap(from_fn(middleware))
            .configure(configure)
            .route(
                "/private",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    let start_login = || {
        test::TestRequest::get()
            .uri("/api/v1/auth/login?return_to=%2Fsettings")
            .to_request()
    };
    let response = test::call_service(&app, start_login()).await;
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/?auth_error=oidc_unavailable"
    );
    let keyed = test::TestRequest::get()
        .uri("/private")
        .insert_header(("x-api-key", "test-api-key"));
    assert_eq!(
        test::call_service(&app, keyed.to_request()).await.status(),
        StatusCode::OK
    );

    provider.available.store(true, Ordering::Relaxed);
    let cached_failure = test::call_service(&app, start_login()).await;
    assert_eq!(
        cached_failure.headers().get(header::LOCATION).unwrap(),
        "/?auth_error=oidc_unavailable"
    );
    // Simulate the retry window expiring without a wall-clock sleep.
    auth.oidc
        .as_ref()
        .unwrap()
        .discovery
        .lock()
        .await
        .refresh_at = Instant::now();

    for subject in ["unlisted", "owner"] {
        let started = test::call_service(&app, start_login()).await;
        assert_eq!(started.status(), StatusCode::SEE_OTHER);
        let url = reqwest::Url::parse(
            started
                .headers()
                .get(header::LOCATION)
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(query["code_challenge_method"], "S256");
        *provider.exchange.lock().unwrap() = Some(Exchange {
            challenge: query["code_challenge"].clone(),
            nonce: query["nonce"].clone(),
            subject,
        });
        let login_cookie = started
            .response()
            .cookies()
            .find(|cookie| cookie.name() == "wyrm_oidc_login")
            .unwrap()
            .into_owned();
        let callback = format!(
            "/api/v1/auth/callback?state={}&code=test-code",
            query["state"]
        );
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&callback)
                .cookie(login_cookie.clone())
                .to_request(),
        )
        .await;
        assert!(provider.exchange.lock().unwrap().is_none());
        let session = response
            .response()
            .cookies()
            .find(|cookie| cookie.name() == "wyrm_session");
        if subject == "unlisted" {
            assert_eq!(
                response.headers().get(header::LOCATION).unwrap(),
                "/?auth_error=oidc"
            );
            assert!(session.is_none());
            continue;
        }
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/settings"
        );
        let session = session
            .expect("a signed token for the owner creates a session")
            .into_owned();
        let request = test::TestRequest::get()
            .uri("/private")
            .cookie(session)
            .to_request();
        assert_eq!(
            test::call_service(&app, request).await.status(),
            StatusCode::OK
        );
        let replay = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&callback)
                .cookie(login_cookie)
                .to_request(),
        )
        .await;
        assert_eq!(
            replay.headers().get(header::LOCATION).unwrap(),
            "/?auth_error=oidc"
        );
        assert!(
            !replay
                .response()
                .cookies()
                .any(|cookie| cookie.name() == "wyrm_session")
        );
    }

    handle.stop(true).await;
    task.await.unwrap().unwrap();
}
