pub(crate) use crate::oidc::tests::configured_auth;
use actix_web::web;
#[cfg(feature = "postgres")]
use database::DatabasePool;
#[cfg(feature = "postgres")]
use wyrm_utils::config::WyrmStartupConfig;
pub(crate) const SESSION_COOKIE: &str = "__Host-wyrm_session";
pub(crate) const LOGIN_COOKIE: &str = "__Host-wyrm_oidc_login";

// Wire handlers directly for unit tests. The routes crate tests registration.
pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/v1/auth/config", web::get().to(crate::config))
        .route("/api/v1/auth/login", web::get().to(crate::oidc::login))
        .route(
            "/api/v1/auth/callback",
            web::get().to(crate::oidc::callback),
        )
        .route(
            "/api/v1/auth/logout",
            web::post().to(crate::session::logout),
        );
}

// The database crate's setup_test_db helper is private to its own unit tests.
#[cfg(feature = "postgres")]
pub(crate) async fn session_pool() -> web::Data<DatabasePool> {
    static MIGRATED: std::sync::Once = std::sync::Once::new();
    let config = WyrmStartupConfig::load().unwrap();
    MIGRATED.call_once(|| {
        let mut conn = database::establish_sync_connection(&config).unwrap();
        database::run_migrations(&mut conn).unwrap();
    });
    web::Data::new(database::create_pool(&config).await.unwrap())
}
