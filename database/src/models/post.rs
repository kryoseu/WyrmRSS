use crate::DatabasePool;
use crate::schema::posts::{self};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use feed_rs::model::Entry;
use serde::Serialize;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(ts_rs::TS)]
#[ts(export)]
pub struct Post {
    pub id: i32,
    pub feed_id: i32,
    pub title: Option<String>,
    pub url: Option<String>,
    pub authors: Option<String>,
    pub published_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub is_favorite: bool,
    pub is_read: bool,
}

impl Post {
    pub async fn get(pool: &DatabasePool, post_id: i32) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        posts::table
            .find(post_id)
            .select(Post::as_select())
            .first(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn create(pool: &DatabasePool, form: PostInsertForm) -> WyrmResult<()> {
        let mut conn = pool.get().await?;
        diesel::insert_into(posts::table)
            .values(form)
            .on_conflict((posts::feed_id, posts::url))
            .do_nothing()
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn update(pool: &DatabasePool, form: PostUpdateForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::update(posts::table.find(form.id))
            .set(form)
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn toggle_is_read(pool: &DatabasePool, post_id: i32) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::update(posts::table.find(post_id))
            .set(posts::is_read.eq(diesel::dsl::not(posts::is_read)))
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)
    }
}

#[derive(Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PostUpdateForm {
    pub id: i32,
    pub is_favorite: Option<bool>,
    pub is_read: Option<bool>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PostInsertForm {
    pub feed_id: i32,
    pub title: Option<String>,
    pub url: Option<String>,
    pub authors: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub content: Option<String>,
}

impl PostInsertForm {
    pub fn from_entry(entry: Entry, feed_id: i32) -> Self {
        let media_description = entry
            .media
            .into_iter()
            .next()
            .and_then(|m| m.description)
            .map(|d| d.content);
        Self {
            feed_id,
            title: entry.title.map(|t| t.content),
            url: entry.links.into_iter().next().map(|l| l.href),
            authors: if entry.authors.is_empty() {
                None
            } else {
                Some(
                    entry
                        .authors
                        .iter()
                        .map(|a| match &a.email {
                            Some(email) => format!("{} ({})", a.name, email),
                            None => a.name.clone(),
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            },
            published_at: entry.published,
            updated_at: entry.updated,
            description: entry.summary.map(|s| s.content).or(media_description),
            content: entry.content.and_then(|c| c.body),
        }
    }
}
