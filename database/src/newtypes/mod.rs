#[cfg(feature = "sqlite")]
use diesel::{backend::Backend, sqlite::Sqlite};
use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{self, Output, ToSql},
    sql_types::Text,
};
#[cfg(feature = "postgres")]
use diesel::{
    pg::{Pg, PgValue},
    sql_types::{Array, Nullable},
};
use diesel_derive_newtype::DieselNewType;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// `text[]` on postgres; a JSON string on sqlite, which has no array type.
#[cfg(feature = "postgres")]
pub type FiltersSql = Array<Nullable<Text>>;
#[cfg(feature = "sqlite")]
pub type FiltersSql = Text;

/// A feed's exclusion filters. Newtype because sqlite has no array type, so
/// storage differs by backend (see `FiltersSql`). `serde(transparent)` keeps
/// the JSON shape identical to a plain `Vec<Option<String>>`.
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
impl ToSql<FiltersSql, Pg> for Filters {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <Vec<Option<String>> as ToSql<FiltersSql, Pg>>::to_sql(&self.0, out)
    }
}

#[cfg(feature = "postgres")]
impl FromSql<FiltersSql, Pg> for Filters {
    fn from_sql(value: PgValue) -> deserialize::Result<Self> {
        <Vec<Option<String>> as FromSql<FiltersSql, Pg>>::from_sql(value).map(Self)
    }
}

#[cfg(feature = "sqlite")]
impl ToSql<FiltersSql, Sqlite> for Filters {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(serde_json::to_string(&self.0)?);
        Ok(serialize::IsNull::No)
    }
}

#[cfg(feature = "sqlite")]
impl FromSql<FiltersSql, Sqlite> for Filters {
    fn from_sql(value: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let raw = <String as FromSql<FiltersSql, Sqlite>>::from_sql(value)?;
        Ok(Self(serde_json::from_str(&raw)?))
    }
}

macro_rules! id_newtype {
    ($name:ident, $doc:literal) => {
        #[derive(
            DieselNewType, Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, TS,
        )]
        #[ts(export)]
        #[doc = $doc]
        pub struct $name(pub i32);
    };
}

id_newtype!(PostId, "The Post ID");
id_newtype!(FeedId, "The Feed ID");
id_newtype!(WebhookId, "The Webhook ID");
id_newtype!(FolderId, "The Folder ID");
