//! Runtime checks for the SQLite backend.
//!
//! The postgres model tests need a live server, so they run against CI's
//! service container. SQLite needs no server, so this drives the real thing
//! end to end against a temp file: pragmas, the JSON `Filters` mapping,
//! `TimestamptzSqlite`, the row-by-row insert path, and FK cascade.
//!
//! Run with:
//!   cargo test -p database --no-default-features --features sqlite
#![cfg(feature = "sqlite")]

use chrono::Utc;
use database::{
    DatabasePool,
    create_pool,
    establish_sync_connection,
    models::{
        feed::{Feed, FeedInsertForm},
        post::{Post, PostInsertForm},
    },
    newtypes::{FeedId, Filters},
    run_migrations,
    schema::posts,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use wyrm_utils::config::WyrmStartupConfig;

/// A migrated database in a uniquely named temp file, plus its pool.
struct TestDb {
    pool: DatabasePool,
    path: String,
}

impl TestDb {
    async fn new(label: &str) -> Self {
        let path = std::env::temp_dir()
            .join(format!(
                "wyrm_smoke_{label}_{}.db",
                Utc::now()
                    .timestamp_nanos_opt()
                    .expect("timestamp in range")
            ))
            .to_string_lossy()
            .into_owned();

        let conf = WyrmStartupConfig {
            database_connection: path.clone(),
            database_pool_size: 4,
            ..Default::default()
        };

        let mut conn = establish_sync_connection(&conf).expect("should open sqlite file");
        run_migrations(&mut conn).expect("should apply migrations");
        drop(conn);

        let pool = create_pool(&conf).await.expect("should build pool");
        Self { pool, path }
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        // WAL mode leaves sidecar files next to the database.
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", self.path));
        }
    }
}

#[derive(QueryableByName)]
struct PragmaI32 {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    value: i32,
}

#[derive(QueryableByName)]
struct PragmaText {
    #[diesel(sql_type = diesel::sql_types::Text)]
    value: String,
}

fn feed_form(url: &str, filters: Option<Filters>) -> FeedInsertForm {
    FeedInsertForm {
        title: "smoke feed".to_string(),
        url: url.to_string(),
        ttl: 60,
        filters,
        ..Default::default()
    }
}

fn post_form(feed_id: FeedId, url: &str) -> PostInsertForm {
    PostInsertForm {
        feed_id,
        title: Some(format!("post {url}")),
        url: Some(url.to_string()),
        authors: None,
        // left to the column default, which is the case that forces the
        // row-by-row insert path on sqlite
        published_at: None,
        updated_at: None,
        description: None,
        content: None,
        created_at: None,
    }
}

/// The pragmas are set per connection by the pool's setup hook. Without
/// `foreign_keys` every ON DELETE clause in the schema is inert.
#[tokio::test]
async fn pool_connections_have_pragmas_applied() {
    let db = TestDb::new("pragmas").await;
    let mut conn = db.pool.get().await.expect("should check out a connection");

    // `PRAGMA x` takes no column alias; the pragma_* table-valued functions do.
    let fk = diesel::sql_query("SELECT foreign_keys AS value FROM pragma_foreign_keys")
        .get_result::<PragmaI32>(&mut conn)
        .await
        .expect("should read foreign_keys");
    assert_eq!(fk.value, 1, "foreign_keys must be ON or cascades no-op");

    let journal = diesel::sql_query("SELECT journal_mode AS value FROM pragma_journal_mode")
        .get_result::<PragmaText>(&mut conn)
        .await
        .expect("should read journal_mode");
    assert_eq!(journal.value.to_lowercase(), "wal");

    // this pragma's column is `timeout`, not `busy_timeout`
    let busy = diesel::sql_query("SELECT timeout AS value FROM pragma_busy_timeout")
        .get_result::<PragmaI32>(&mut conn)
        .await
        .expect("should read busy_timeout");
    assert_eq!(busy.value, 5000);
}

/// `filters` is `text[]` on postgres and a JSON string on sqlite; the models
/// name one type either way, so the mapping has to survive a round trip.
#[tokio::test]
async fn filters_round_trip_through_json() {
    let db = TestDb::new("filters").await;
    let filters = Filters(vec![Some("url:ads".to_string()), Some("spam".to_string())]);

    let created = Feed::create(&db.pool, feed_form("https://example.com/a", Some(filters)))
        .await
        .expect("should create feed");
    assert_eq!(
        created.filters.0,
        vec![Some("url:ads".to_string()), Some("spam".to_string())],
        "filters should survive the insert"
    );

    // Re-read so the value comes back through FromSql rather than RETURNING.
    let fetched = Feed::get(&db.pool, created.id).await.expect("should get");
    assert_eq!(fetched.filters.0, created.filters.0);

    // The column default is the JSON literal '[]', not NULL.
    let empty = Feed::create(&db.pool, feed_form("https://example.com/b", None))
        .await
        .expect("should create feed without filters");
    assert!(
        empty.filters.0.is_empty(),
        "default should be an empty list"
    );
}

/// Timestamps are TEXT under sqlite; the strftime defaults in the migration
/// have to parse back as UTC-aware values via `TimestamptzSqlite`.
#[tokio::test]
async fn timestamp_defaults_read_back_as_utc() {
    let db = TestDb::new("timestamps").await;
    let before = Utc::now();

    let feed = Feed::create(&db.pool, feed_form("https://example.com/ts", None))
        .await
        .expect("should create feed");

    let after = Utc::now();
    assert!(
        feed.created_at >= before - chrono::Duration::seconds(5)
            && feed.created_at <= after + chrono::Duration::seconds(5),
        "created_at {} should sit near now ({before} .. {after})",
        feed.created_at
    );
    assert!(feed.last_fetched_at.is_none());
}

/// `create_many` inserts row by row on sqlite. It must still honour
/// ON CONFLICT DO NOTHING and return only the rows actually inserted.
#[tokio::test]
async fn create_many_inserts_and_skips_conflicts() {
    let db = TestDb::new("create_many").await;
    let feed = Feed::create(&db.pool, feed_form("https://example.com/many", None))
        .await
        .expect("should create feed");

    let inserted = Post::create_many(
        &db.pool,
        vec![
            post_form(feed.id, "https://example.com/p1"),
            post_form(feed.id, "https://example.com/p2"),
        ],
    )
    .await
    .expect("should insert posts");
    assert_eq!(inserted.len(), 2);

    // p1 duplicates an existing (feed_id, url); p3 is new.
    let second = Post::create_many(
        &db.pool,
        vec![
            post_form(feed.id, "https://example.com/p1"),
            post_form(feed.id, "https://example.com/p3"),
        ],
    )
    .await
    .expect("should insert posts");
    assert_eq!(
        second.len(),
        1,
        "the conflicting row should be skipped, not error"
    );
    assert_eq!(second[0].url.as_deref(), Some("https://example.com/p3"));

    // published_at came from the column default, so it must have parsed.
    assert!(second[0].published_at <= Utc::now() + chrono::Duration::seconds(5));
}

/// Proves the pragma actually has teeth: deleting a feed removes its posts.
#[tokio::test]
async fn deleting_a_feed_cascades_to_posts() {
    let db = TestDb::new("cascade").await;
    let feed = Feed::create(&db.pool, feed_form("https://example.com/cascade", None))
        .await
        .expect("should create feed");
    Post::create_many(&db.pool, vec![post_form(feed.id, "https://example.com/c1")])
        .await
        .expect("should insert post");

    let mut conn = db.pool.get().await.expect("should check out a connection");
    let before: i64 = posts::table
        .filter(posts::feed_id.eq(feed.id))
        .count()
        .get_result(&mut conn)
        .await
        .expect("should count posts");
    assert_eq!(before, 1);

    Feed::delete(&db.pool, feed.id)
        .await
        .expect("should delete feed");

    let after: i64 = posts::table
        .filter(posts::feed_id.eq(feed.id))
        .count()
        .get_result(&mut conn)
        .await
        .expect("should count posts");
    assert_eq!(after, 0, "posts should cascade away with their feed");
}
