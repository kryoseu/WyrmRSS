use std::time::Duration;

use crate::http::HttpClient;
use chrono::Utc;
use database::{
    DatabasePool,
    models::{
        feed::{Feed, FeedUpdateForm},
        post::{Post, PostInsertForm},
    },
};
use futures::future::join_all;
use tracing::{error, info};
use utils::result::WyrmResult;

const INTERVAL: Duration = Duration::from_secs(30);

/// Background worker responsible for polling RSS feeds on a scheduled interval.
///
/// The worker runs indefinitely, waking every [`INTERVAL`] seconds to check
/// which feeds are due for polling and fetching new posts from them.
pub struct FeedWorker {
    pub db_pool: DatabasePool,
    pub http: HttpClient,
}

impl FeedWorker {
    /// Creates a new [`FeedWorker`] with the given database pool.
    pub fn new(pool: DatabasePool, http: HttpClient) -> Self {
        Self {
            db_pool: pool,
            http,
        }
    }

    /// Starts the feed polling loop.
    ///
    /// Runs indefinitely, polling due feeds every [`INTERVAL`] seconds.
    /// Errors are logged but do not stop the worker.
    pub async fn run(&mut self) {
        let mut interval = tokio::time::interval(INTERVAL);
        loop {
            interval.tick().await;
            if let Err(e) = self.poll_feeds().await {
                error!("Feed worker error: {e}");
            }
        }
    }

    /// Queries the database for feeds due for polling and fetches new posts.
    async fn poll_feeds(&mut self) -> WyrmResult<()> {
        let feeds = Feed::get_all(&self.db_pool).await?;

        let due_feeds: Vec<Feed> = feeds.into_iter().filter(|f| f.is_due()).collect();

        let tasks: Vec<_> = due_feeds
            .into_iter()
            .map(|feed| {
                let http = self.http.clone();
                let pool = self.db_pool.clone();
                tokio::spawn(async move {
                    if let Err(e) = process_feed(&http, &pool, &feed).await {
                        error!("Failed to process feed {}: {e}", feed.url);
                    }
                })
            })
            .collect();

        join_all(tasks).await;

        Ok(())
    }

    // async fn fetch_feed(&self, feed: Feed) -> WyrmResult<()> { ... }
}

/// Fetch -> Parse -> Store -> Update
async fn process_feed(http: &HttpClient, pool: &DatabasePool, feed: &Feed) -> WyrmResult<()> {
    info!("Processing feed {}", feed.title);

    let bytes = http.fetch(&feed.url).await?;

    let parsed = feed_rs::parser::parse(&bytes[..])?;

    for entry in parsed.entries {
        let url = entry.links.first().map(|l| l.href.as_str()).unwrap_or("");

        let filtered = feed
            .url_filter
            .iter()
            .filter_map(|f| f.as_deref())
            .any(|f| url.contains(f));

        if filtered {
            continue;
        }

        if let Err(e) = Post::create(pool, PostInsertForm::from_entry(entry, feed.id)).await {
            error!("Failed to insert post: {e}");
        }
    }

    Feed::update(
        pool,
        FeedUpdateForm {
            id: feed.id,
            title: None,
            url: None,
            ttl: None,
            url_filter: None,
            last_fetched_at: Some(Utc::now()),
        },
    )
    .await?;

    Ok(())
}
