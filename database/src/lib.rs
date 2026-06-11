pub mod models;
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
    let conn = PgConnection::establish(&conf.database.connection)?;
    Ok(conn)
}

pub async fn create_pool(conf: &WyrmStartupConfig) -> WyrmResult<DatabasePool> {
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&conf.database.connection);
    Pool::builder(config)
        .build()
        .map_err(|e| WyrmError::Database(DatabaseError::PoolBuildError(e)))
}
