use super::*;
use crate::tests::{SESSION_COOKIE, configure, configured_auth};
#[cfg(feature = "postgres")]
use crate::{
    session::{SESSION_IDLE_TIMEOUT, token_hash},
    tests::session_pool,
};
#[cfg(feature = "postgres")]
use actix_web::http::header;
use actix_web::{App, HttpResponse, cookie::Cookie, http::StatusCode, middleware::from_fn, test};
#[cfg(feature = "postgres")]
use chrono::{TimeDelta, Utc};
#[cfg(feature = "postgres")]
use database::models::session::Session;
#[cfg(feature = "postgres")]
use openidconnect::CsrfToken;
use wyrm_utils::config::WyrmStartupConfig;

#[actix_web::test]
async fn api_key_access_needs_no_session_storage() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(configured_auth()))
            .wrap(from_fn(middleware))
            .route("/protected", web::post().to(HttpResponse::Ok)),
    )
    .await;
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/protected")
            .insert_header(("x-api-key", "api-secret"))
            .cookie(Cookie::new(SESSION_COOKIE, "stale-session"))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        !response
            .response()
            .cookies()
            .any(|c| c.name() == SESSION_COOKIE)
    );
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
#[cfg(feature = "postgres")]
async fn api_key_is_independent_of_oidc_and_sessions_require_csrf() {
    let pool = session_pool().await;
    let auth = web::Data::new(configured_auth());
    let session_token = CsrfToken::new_random().secret().clone();
    let now = Utc::now();
    Session::create(
        &pool,
        Session {
            token_hash: token_hash(&session_token),
            expires_at: now + SESSION_IDLE_TIMEOUT,
            last_seen_at: now,
        },
        None,
    )
    .await
    .unwrap();
    let app = test::init_service(
        App::new()
            .app_data(auth.clone())
            .app_data(pool.clone())
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
        (session_token.as_str(), None, None, StatusCode::FORBIDDEN),
        (
            session_token.as_str(),
            Some("1"),
            Some("https://evil.example"),
            StatusCode::FORBIDDEN,
        ),
        (
            session_token.as_str(),
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
        .cookie(Cookie::new(SESSION_COOKIE, session_token.clone()))
        .to_request();
    assert_eq!(
        test::call_service(&app, request).await.status(),
        StatusCode::OK
    );
    let request = test::TestRequest::post()
        .uri("/api/v1/auth/logout")
        .insert_header(("x-wyrm-csrf", "1"))
        .cookie(Cookie::new(SESSION_COOKIE, session_token.clone()))
        .to_request();
    assert_eq!(
        test::call_service(&app, request).await.status(),
        StatusCode::NO_CONTENT
    );
    assert!(
        Session::refresh(
            &pool,
            &token_hash(&session_token),
            now,
            now + SESSION_IDLE_TIMEOUT
        )
        .await
        .unwrap()
        .is_none()
    );
}

#[actix_web::test]
#[cfg(feature = "postgres")]
async fn oidc_only_is_protected_and_expired_sessions_are_rejected() {
    let pool = session_pool().await;
    let expired_token = CsrfToken::new_random().secret().clone();
    let mut auth = configured_auth();
    auth.api_key = None;
    let now = Utc::now();
    Session::create(
        &pool,
        Session {
            token_hash: token_hash(&expired_token),
            expires_at: now - TimeDelta::seconds(1),
            last_seen_at: now - SESSION_IDLE_TIMEOUT,
        },
        None,
    )
    .await
    .unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(auth))
            .app_data(pool.clone())
            .wrap(from_fn(middleware))
            .configure(configure)
            .route("/protected", web::get().to(HttpResponse::Ok)),
    )
    .await;
    for token in [expired_token.as_str(), "forged"] {
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
