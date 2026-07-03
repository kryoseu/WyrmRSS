// `macro_use` re-exports the test fixture macros (`test_feed!`/`test_post!`)
// declared inside `models` so the `views` tests can use them too.
#[macro_use]
pub mod models;
pub mod newtypes;
pub mod schema;
pub mod utils;
pub mod views;
use diesel::{Connection, PgConnection};
use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::error::Error;
use wyrm_utils::{
    config::WyrmStartupConfig,
    error::{DatabaseError, WyrmError},
    result::WyrmResult,
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations/");

pub type DatabasePool = Pool<AsyncPgConnection>;

pub fn run_migrations(
    connection: &mut PgConnection,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    connection.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}

pub fn establish_sync_connection(conf: &WyrmStartupConfig) -> WyrmResult<PgConnection> {
    let conn = PgConnection::establish(&conf.database_connection)?;
    Ok(conn)
}

pub async fn create_pool(conf: &WyrmStartupConfig) -> WyrmResult<DatabasePool> {
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&conf.database_connection);
    Pool::builder(config)
        .max_size(conf.database_pool_size)
        .build()
        .map_err(|e| WyrmError::Database(DatabaseError::PoolBuildError(e)))
}

/// Builds a connection pool against the configured test database, ensuring the
/// schema is migrated first.
///
/// Migrations are applied at most once per test process (via [`Once`]), so it's
/// cheap for every database test to call this — the first call sets up the
/// schema, later calls just hand back a fresh pool. Applying the embedded
/// migrations mirrors what the server does on startup, so a blank database
/// (e.g. CI's Postgres service) gets set up the same way `diesel migration run`
/// would, without needing the diesel CLI installed.
#[cfg(test)]
pub(crate) async fn setup_test_db() -> DatabasePool {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        // `load()` resolves config/wyrm.toml relative to the cwd, but tests run
        // from the crate dir — point at the workspace root so we read the same
        // config the server uses. Done once, before any `load()` below.
        std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
            .expect("should cd to workspace root");

        let conf = WyrmStartupConfig::load().expect("should load test config");
        let mut conn = establish_sync_connection(&conf).expect("should connect for migrations");
        run_migrations(&mut conn).expect("should run migrations");
    });

    let conf = WyrmStartupConfig::load().expect("should load test config");
    create_pool(&conf)
        .await
        .expect("should build pool against the test database")
}
