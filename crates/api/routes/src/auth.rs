use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").route(
        "/verify",
        web::get().to(|| async { actix_web::HttpResponse::Ok().finish() }),
    ));
}

/// Server-only routes; the desktop app keeps its API-key authentication.
#[cfg(feature = "auth")]
pub fn server_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/v1/auth/config", web::get().to(auth::config))
        .route("/api/v1/auth/login", web::get().to(auth::oidc::login))
        .route("/api/v1/auth/callback", web::get().to(auth::oidc::callback))
        .route("/api/v1/auth/logout", web::post().to(auth::session::logout));
}

#[cfg(all(test, feature = "auth"))]
mod tests {
    use super::*;
    use actix_web::{
        App,
        http::{StatusCode, header},
        middleware::from_fn,
        test,
    };
    use auth::{Auth, middleware::middleware};
    use wyrm_utils::config::WyrmStartupConfig;

    #[actix_web::test]
    async fn server_registers_auth_routes_alongside_api_key_verification() {
        let auth = Auth::new(&WyrmStartupConfig {
            api_key: Some("key".into()),
            oidc_issuer_url: Some("https://id.example.com".into()),
            oidc_client_id: Some("wyrm".into()),
            oidc_redirect_url: Some("https://rss.example.com/api/v1/auth/callback".into()),
            ..Default::default()
        })
        .unwrap();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(auth))
                .wrap(from_fn(middleware))
                .configure(server_config)
                .configure(crate::config),
        )
        .await;
        let config: serde_json::Value = test::call_and_read_body_json(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/auth/config")
                .to_request(),
        )
        .await;
        assert_eq!(
            config,
            serde_json::json!({
                "oidc_enabled": true, "api_key_enabled": true, "oidc_session": false,
            })
        );
        // Missing callback state is handled without contacting the provider.
        let callback = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/auth/callback")
                .to_request(),
        )
        .await;
        assert_eq!(callback.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            callback.headers().get(header::LOCATION).unwrap(),
            "/?auth_error=oidc"
        );
        let logout = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/v1/auth/logout")
                .insert_header(("x-wyrm-csrf", "1"))
                .to_request(),
        )
        .await;
        assert_eq!(logout.status(), StatusCode::NO_CONTENT);
        let verify = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/auth/verify")
                .insert_header(("x-api-key", "key"))
                .to_request(),
        )
        .await;
        assert_eq!(verify.status(), StatusCode::OK);
    }
}
