use crate::http::{HttpClient, HttpConfig};
use chrono::Utc;
use database::{
    DatabasePool,
    models::{
        feed::{Feed, FeedUpdateForm},
        post::{Post, PostInsertForm},
    },
    utils::settings::RuntimeSettings,
};
use futures::future::join_all;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::Receiver;
use tracing::{error, info};
use wyrm_utils::result::WyrmResult;

#[derive(Debug)]
pub enum WorkerCommand {
    /// Trigger an immediate feed poll.
    /// The sender is notified when polling completes.
    PollFeeds(tokio::sync::oneshot::Sender<()>),
    /// Reload runtime settings (e.g. HTTP client config)
    /// without restarting the worker.
    Reconfigure,
}

/// Polls due feeds every `feed_poll_interval_secs`, spawning a concurrent task per feed.
/// Accepts [`WorkerCommand`]s to trigger an immediate poll or apply updated runtime
/// settings without waiting for the current interval to expire.
pub struct FeedWorker {
    pub db_pool: DatabasePool,
    pub http: HttpClient,
    pub runtime_settings: Arc<RwLock<RuntimeSettings>>,
}

impl FeedWorker {
    pub fn new(
        pool: DatabasePool,
        http: HttpClient,
        runtime_settings: Arc<RwLock<RuntimeSettings>>,
    ) -> Self {
        Self {
            db_pool: pool,
            http,
            runtime_settings,
        }
    }

    /// Starts the feed polling loop.
    pub async fn run(&mut self, mut rx: Receiver<WorkerCommand>) -> WyrmResult<()> {
        loop {
            let interval = {
                let secs = self.runtime_settings.read()?.feed_poll_interval_secs;
                std::time::Duration::from_secs(secs as u64)
            };
            tokio::select! {
                _ = tokio::time::sleep(interval) => {
                    if let Err(e) = self.poll_feeds().await {
                        error!("Feed worker error: {e}");
                    }
                }
                Some(cmd) = rx.recv() => {
                    match cmd {
                        WorkerCommand::PollFeeds(reply) => {
                            if let Err(e) = self.poll_feeds().await {
                                error!("Feed worker error: {e}");
                            }
                            let _ = reply.send(());
                        }
                        WorkerCommand::Reconfigure => {
                            let rs = self.runtime_settings.read()?;
                            self.http = HttpClient::new(&HttpConfig::from(&*rs))?;
                        }
                    }
                }
            }
        }
    }

    /// Queries the database for feeds due for polling and fetches new posts.
    async fn poll_feeds(&mut self) -> WyrmResult<()> {
        let feeds = Feed::get_all(&self.db_pool).await?;

        let due_feeds: Vec<Feed> = feeds.into_iter().filter(|f| f.is_due()).collect();

        info!("Processing {} due feeds", due_feeds.len());

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
            tag: None,
            tag_color: None,
            url_filter: None,
            last_fetched_at: Some(Utc::now()),
        },
    )
    .await?;

    Ok(())
}
