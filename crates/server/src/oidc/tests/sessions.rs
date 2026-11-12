use super::*;

// The database crate's setup_test_db helper is private to its own unit tests.
// Server tests use the same configured PostgreSQL database and migrate once.
pub(super) async fn session_pool() -> web::Data<DatabasePool> {
    static MIGRATED: std::sync::Once = std::sync::Once::new();
    let config = WyrmStartupConfig::load().unwrap();
    MIGRATED.call_once(|| {
        let mut conn = database::establish_sync_connection(&config).unwrap();
        database::run_migrations(&mut conn).unwrap();
    });
    web::Data::new(database::create_pool(&config).await.unwrap())
}

#[actix_web::test]
async fn activity_renews_cookie_and_survives_auth_restart() {
    let pool = session_pool().await;
    let token = CsrfToken::new_random().secret().clone();
    let issued_at = Utc::now() - TimeDelta::hours(11);
    Session::create(
        &pool,
        Session {
            token_hash: token_hash(&token),
            expires_at: issued_at + SESSION_IDLE_TIMEOUT,
            last_seen_at: issued_at,
        },
        None,
    )
    .await
    .unwrap();

    for path in ["/protected", "/api/v1/auth/config"] {
        // A fresh Auth instance authenticates using the persisted session.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(configured_auth()))
                .app_data(pool.clone())
                .wrap(from_fn(middleware))
                .configure(configure)
                .route("/protected", web::get().to(HttpResponse::Ok)),
        )
        .await;
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(path)
                .cookie(Cookie::new(SESSION_COOKIE, token.clone()))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let cookie = response
            .response()
            .cookies()
            .find(|c| c.name() == SESSION_COOKIE)
            .unwrap();
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
        assert!(
            cookie.max_age().unwrap().whole_seconds() >= SESSION_IDLE_TIMEOUT.num_seconds() - 2
        );
        if path == "/api/v1/auth/config" {
            let config: serde_json::Value = test::read_body_json(response).await;
            assert_eq!(config["oidc_session"], true);
        }
    }
    Session::delete(&pool, &token_hash(&token)).await.unwrap();
}
