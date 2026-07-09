use crate::{DatabaseConn, DatabasePool, newtypes::FolderId, schema::folders};
use diesel::{define_sql_function, prelude::*};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

define_sql_function! {
    /// Postgres LOWER(), used for case-insensitive folder lookups so casing
    /// rules match the unique index on LOWER(name).
    fn lower(x: diesel::sql_types::Text) -> diesel::sql_types::Text;
}

#[serde_with::skip_serializing_none]
#[derive(Clone, Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::folders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(ts_rs::TS)]
#[ts(optional_fields, export)]
pub struct Folder {
    pub id: FolderId,
    pub name: String,
}

impl Folder {
    pub async fn get(pool: &DatabasePool, id: FolderId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        folders::table
            .find(id)
            .select(Folder::as_select())
            .first(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn get_all(pool: &DatabasePool) -> WyrmResult<Vec<Self>> {
        let mut conn = pool.get().await?;
        folders::table
            .select(Self::as_select())
            .load(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    /// Finds a folder by name (ignoring case and surrounding whitespace) or
    /// creates it. If the folder exists, its stored spelling is kept — the
    /// caller's casing does not overwrite it. Blank names are rejected.
    pub async fn resolve_or_create(pool: &DatabasePool, name: &str) -> WyrmResult<Self> {
        let conn = pool.get().await?;
        Self::resolve_or_create_on(&conn, name).await
    }

    /// Like [`resolve_or_create`](Self::resolve_or_create), but runs on an
    /// existing connection so it can join a caller's transaction.
    pub async fn resolve_or_create_on(mut conn: &DatabaseConn, name: &str) -> WyrmResult<Self> {
        let name = normalize_name(name)?;

        let existing = folders::table
            .filter(lower(folders::name).eq(lower(name)))
            .select(Self::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(WyrmError::from)?;

        match existing {
            Some(folder) => Ok(folder),
            None => diesel::insert_into(folders::table)
                .values(FolderInsertForm { name: name.into() })
                .get_result(&mut conn)
                .await
                .map_err(WyrmError::from),
        }
    }

    /// Resolves a user-supplied folder name to an id on `conn`, creating the
    /// folder when the name is new. This is what lets feed writes take a
    /// folder *name* instead of an id, so callers just pass along whatever
    /// the user (or an OPML outline) typed. A blank or absent name resolves
    /// to `None` (no folder) rather than erroring.
    pub(crate) async fn resolve_name_on(
        conn: &DatabaseConn,
        name: Option<&str>,
    ) -> WyrmResult<Option<FolderId>> {
        match name.map(str::trim).filter(|n| !n.is_empty()) {
            Some(name) => Ok(Some(Self::resolve_or_create_on(conn, name).await?.id)),
            None => Ok(None),
        }
    }

    pub async fn update(pool: &DatabasePool, mut form: FolderUpdateForm) -> WyrmResult<Self> {
        form.name = normalize_name(&form.name)?.to_string();
        let mut conn = pool.get().await?;
        diesel::update(folders::table.find(form.id))
            .set(form)
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn delete(pool: &DatabasePool, id: FolderId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::delete(folders::table.find(id))
            .returning(Self::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(WyrmError::from)
    }
}

/// Trims a folder name, rejecting one that is empty after trimming. Every
/// folder write goes through this: the LOWER(name) unique index has no TRIM,
/// so an untrimmed insert would slip past the case-insensitive dedup.
fn normalize_name(name: &str) -> WyrmResult<&str> {
    let name = name.trim();
    if name.is_empty() {
        return Err(WyrmError::EmptyFolderName);
    }
    Ok(name)
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::folders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FolderInsertForm {
    pub name: String,
}

#[derive(Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::folders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FolderUpdateForm {
    pub id: FolderId,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup_test_db;

    /// Unique folder name per run — the test database is shared across
    /// concurrently running tests.
    fn unique_name(prefix: &str) -> String {
        format!(
            "{prefix} {}",
            chrono::Utc::now()
                .timestamp_nanos_opt()
                .expect("timestamp in range")
        )
    }

    #[tokio::test]
    async fn resolve_or_create_trims_and_reuses_case_insensitively() {
        let pool = setup_test_db().await;
        let name = unique_name("YouTube");

        let created = Folder::resolve_or_create(&pool, &format!("  {name} "))
            .await
            .expect("create should succeed");
        assert_eq!(created.name, name, "name should be stored trimmed");

        let resolved = Folder::resolve_or_create(&pool, &name.to_lowercase())
            .await
            .expect("resolve should succeed");
        assert_eq!(resolved.id, created.id, "case variant should reuse the row");
        assert_eq!(resolved.name, name, "stored casing should win");

        Folder::delete(&pool, created.id)
            .await
            .expect("should delete folder");
    }

    #[tokio::test]
    async fn resolve_or_create_rejects_blank_name() {
        let pool = setup_test_db().await;
        assert!(
            matches!(
                Folder::resolve_or_create(&pool, "   ").await,
                Err(WyrmError::EmptyFolderName)
            ),
            "blank name should be rejected"
        );
    }

    #[tokio::test]
    async fn update_trims_renamed_folder() {
        let pool = setup_test_db().await;
        let folder = Folder::resolve_or_create(&pool, &unique_name("News"))
            .await
            .expect("create should succeed");

        let new_name = unique_name("Tech");
        let renamed = Folder::update(
            &pool,
            FolderUpdateForm {
                id: folder.id,
                name: format!("  {new_name} "),
            },
        )
        .await
        .expect("rename should succeed");
        assert_eq!(renamed.id, folder.id);
        assert_eq!(renamed.name, new_name, "rename should be stored trimmed");

        Folder::delete(&pool, folder.id)
            .await
            .expect("should delete folder");
    }
}
