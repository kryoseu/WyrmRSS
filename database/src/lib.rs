pub mod models;
pub mod schema;
pub mod views;
use diesel::{Connection, PgConnection};
use std::error::Error;
use utils::error::{DatabaseError, WyrmError};
use utils::settings::WyrmSettings;

use diesel_async::AsyncPgConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;
use utils::result::WyrmResult;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations/");

pub type DatabasePool = Pool<AsyncPgConnection>;

pub fn run_migrations(
    connection: &mut PgConnection,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    connection.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}

pub fn establish_sync_connection(settings: &WyrmSettings) -> WyrmResult<PgConnection> {
    let conn = PgConnection::establish(&settings.database.connection)?;
    Ok(conn)
}

pub async fn create_pool(settings: &WyrmSettings) -> WyrmResult<DatabasePool> {
    let config =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new(&settings.database.connection);
    Pool::builder(config)
        .build()
        .map_err(|e| WyrmError::Database(DatabaseError::PoolBuildError(e)))
}

// use diesel_async::AsyncConnection;
// pub async fn transaction(pool: &DatabasePool, feed_id: i32) -> WyrmResult<()> {
//     let mut conn = pool.get().await?;
//     let conn = &mut *conn;
//     conn.transaction(async move |conn| {
//         diesel::delete(posts::table.filter(posts::feed_id.eq(feed_id)))
//             .execute(conn)
//             .await?;
//         diesel::delete(feeds::table.find(feed_id))
//             .execute(conn)
//             .await?;
//         Ok::<(), diesel::result::Error>(())
//     })
//     .await
//     .map_err(WyrmError::from)
// }
