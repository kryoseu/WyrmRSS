use diesel::prelude::*;
use diesel::{Selectable, deserialize::Queryable};
use diesel_async::RunQueryDsl;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

use crate::schema::local_instance;
use crate::{DatabaseConn, DatabasePool, schema};

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::local_instance)]
#[diesel(check_for_backend(crate::Backend))]
pub struct LocalInstance {
    /// Whether this instance has been set up yet.
    /// True from the start on desktop, which is single-user and skips setup.
    pub is_setup: bool,
}

impl LocalInstance {
    pub async fn get(pool: &DatabasePool) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        local_instance::table
            .select(LocalInstance::as_select())
            .first(&mut conn)
            .await
            .map_err(WyrmError::from)
    }
    /// Marks the instance as set up; false if it already was.
    pub async fn claim_setup(conn: &mut DatabaseConn) -> WyrmResult<bool> {
        let updated =
            diesel::update(local_instance::table.filter(local_instance::is_setup.eq(false)))
                .set(local_instance::is_setup.eq(true))
                .execute(conn)
                .await?;
        Ok(updated == 1)
    }
}
