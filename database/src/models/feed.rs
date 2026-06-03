use crate::{DatabasePool, schema::feeds};
use chrono::{DateTime, Utc};
use diesel::{
    Selectable,
    prelude::{Queryable, *},
};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::feeds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(ts_rs::TS)]
#[ts(optional_fields, export)]
pub struct Feed {
    pub id: i32,
    pub title: String,
    pub url: String,
    pub ttl: i32,
    pub tag: Option<String>,
    pub tag_color: Option<String>,
    pub url_filter: Vec<Option<String>>,
    pub last_fetched_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Feed {
    pub async fn get(pool: &DatabasePool, feed_id: i32) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        feeds::table
            .find(feed_id)
            .select(Feed::as_select())
            .first(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn get_all(pool: &DatabasePool) -> WyrmResult<Vec<Self>> {
        let mut conn = pool.get().await?;
        feeds::table
            .select(Self::as_select())
            .load(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn create(pool: &DatabasePool, form: FeedInsertForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::insert_into(feeds::table)
            .values(form)
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn update(pool: &DatabasePool, form: FeedUpdateForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::update(feeds::table.find(form.id))
            .set(form)
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn delete(pool: &DatabasePool, feed_id: i32) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::delete(feeds::table.find(feed_id))
            .returning(Self::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub fn is_due(&self) -> bool {
        match self.last_fetched_at {
            None => true,
            Some(last_fetched_at) => {
                let elapsed = Utc::now() - last_fetched_at;
                elapsed.num_minutes() >= self.ttl as i64
            }
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::feeds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FeedInsertForm {
    pub title: String,
    pub url: String,
    pub ttl: i32,
    pub tag: Option<String>,
    pub tag_color: Option<String>,
    pub url_filter: Option<Vec<Option<String>>>,
}

#[derive(Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::feeds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FeedUpdateForm {
    pub id: i32,
    pub title: Option<String>,
    pub url: Option<String>,
    pub ttl: Option<i32>,
    pub tag: Option<String>,
    pub tag_color: Option<String>,
    pub url_filter: Option<Vec<Option<String>>>,
    pub last_fetched_at: Option<DateTime<Utc>>,
}
