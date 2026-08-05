use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{self, Output, ToSql},
};
use diesel_derive_newtype::DieselNewType;
use serde::{Deserialize, Serialize};

/// `text[]` on postgres; a JSON string on sqlite, which has no array type.
#[cfg(feature = "postgres")]
pub type FiltersSql =
    diesel::sql_types::Array<diesel::sql_types::Nullable<diesel::sql_types::Text>>;
#[cfg(feature = "sqlite")]
pub type FiltersSql = diesel::sql_types::Text;

/// A feed's exclusion filters.
///
/// A newtype only because the orphan rule blocks implementing diesel's
/// `ToSql`/`FromSql` on a bare `Vec<Option<String>>`. `serde(transparent)`
/// keeps the JSON identical to the plain vector, and the `Deref`/`FromIterator`
/// impls below mean callers outside this crate read and build filters exactly
/// as they did before.
///
/// Deliberately does not derive `ts_rs::TS`: that would emit a named `Filters`
/// alias and make every struct holding one import it, changing the generated
/// frontend types. The exporting structs override the field type instead.
#[derive(Clone, Debug, Default, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = FiltersSql)]
#[serde(transparent)]
pub struct Filters(pub Vec<Option<String>>);

/// Keeps `&feed.filters` coercing to `&[Option<String>]` for readers.
impl std::ops::Deref for Filters {
    type Target = Vec<Option<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Lets the existing `.map(Some).collect()` chains in `api_crud` build a
/// `Filters` without naming it.
impl FromIterator<Option<String>> for Filters {
    fn from_iter<I: IntoIterator<Item = Option<String>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl From<Vec<Option<String>>> for Filters {
    fn from(value: Vec<Option<String>>) -> Self {
        Self(value)
    }
}

#[cfg(feature = "postgres")]
impl ToSql<FiltersSql, diesel::pg::Pg> for Filters {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::pg::Pg>) -> serialize::Result {
        <Vec<Option<String>> as ToSql<FiltersSql, diesel::pg::Pg>>::to_sql(&self.0, out)
    }
}

#[cfg(feature = "postgres")]
impl FromSql<FiltersSql, diesel::pg::Pg> for Filters {
    fn from_sql(value: diesel::pg::PgValue) -> deserialize::Result<Self> {
        <Vec<Option<String>> as FromSql<FiltersSql, diesel::pg::Pg>>::from_sql(value).map(Self)
    }
}

#[cfg(feature = "sqlite")]
impl ToSql<FiltersSql, diesel::sqlite::Sqlite> for Filters {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::sqlite::Sqlite>) -> serialize::Result {
        out.set_value(serde_json::to_string(&self.0)?);
        Ok(serialize::IsNull::No)
    }
}

#[cfg(feature = "sqlite")]
impl FromSql<FiltersSql, diesel::sqlite::Sqlite> for Filters {
    fn from_sql(
        value: <diesel::sqlite::Sqlite as diesel::backend::Backend>::RawValue<'_>,
    ) -> deserialize::Result<Self> {
        let raw = <String as FromSql<FiltersSql, diesel::sqlite::Sqlite>>::from_sql(value)?;
        Ok(Self(serde_json::from_str(&raw)?))
    }
}

#[derive(
    DieselNewType, Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS,
)]
#[ts(export)]
/// The Post ID
pub struct PostId(pub i32);

#[derive(
    DieselNewType, Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS,
)]
#[ts(export)]
/// The Feed ID
pub struct FeedId(pub i32);

#[derive(
    DieselNewType, Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS,
)]
#[ts(export)]
/// The Webhook ID
pub struct WebhookId(pub i32);

#[derive(
    DieselNewType, Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS,
)]
#[ts(export)]
/// The Folder ID
pub struct FolderId(pub i32);
