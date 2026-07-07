use crate::{
    filter::CompiledFilters,
    http::{HttpClient, HttpConfig},
};
use chrono::Utc;
use database::{
    DatabasePool,
    models::{
        feed::{Feed, FeedUpdateForm},
        post::{Post, PostInsertForm},
        webhook::Webhook,
    },
    utils::settings::RuntimeSettings,
    views,
};
use futures::future::join_all;
use std::sync::{Arc, RwLock};
use tokio::sync::{Semaphore, mpsc::Receiver};
use tracing::{error, info, warn};
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
                        WorkerCommand::Reconfigure => {}  // interrupts the sleep; interval re-read at loop top
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

        // Re-configure the http client from in-memory runtime settings
        // as settings may have been updated.
        self.http = HttpClient::new(&HttpConfig::from(&*self.runtime_settings.read()?))?;

        let webhooks = views::webhook::all_by_feed(&self.db_pool).await?;

        // Each in-flight feed holds ~1 pool connection. Reserve headroom so polling
        // can never starve the API handlers of the shared pool.
        let permits = (self.db_pool.status().max_size / 2).max(1);
        let semaphore = Arc::new(Semaphore::new(permits));

        let tasks: Vec<_> = due_feeds
            .into_iter()
            .map(|feed| {
                let http = self.http.clone();
                let pool = self.db_pool.clone();
                let feed_webhooks = webhooks.get(&feed.id).cloned().unwrap_or_default();
                let semaphore = semaphore.clone();
                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    if let Err(e) = process_feed(&pool, &http, &feed, &feed_webhooks).await {
                        error!("Failed to process feed {}: {e}", feed.url);
                    }
                })
            })
            .collect();

        join_all(tasks).await;

        Ok(())
    }
}

/// Fetch -> Parse -> Store -> Update
async fn process_feed(
    pool: &DatabasePool,
    http: &HttpClient,
    feed: &Feed,
    webhooks: &[Webhook],
) -> WyrmResult<()> {
    info!("Processing feed {}", feed.title);

    let bytes = http.fetch(&feed.url).await?;

    let parsed = feed_rs::parser::parse(&bytes[..])?;

    let filters = CompiledFilters::new(&feed.filters);

    let mut forms: Vec<PostInsertForm> = parsed
        .entries
        .into_iter()
        .filter(|entry| !filters.excludes(entry))
        .map(|entry| PostInsertForm::from_entry(entry, feed.id))
        .collect();

    // Insert oldest-first so serial ids ascend with publish date: post lists
    // sort by (created_at, id) and a whole batch shares one created_at, so
    // insertion order is the display order within the batch. Entries without
    // a publish date get NOW() from the column default, hence sort as newest.
    let now = Utc::now();
    forms.sort_by_key(|f| f.published_at.unwrap_or(now));

    // A feed's first fetch is backfill, not news: stamp created_at with the
    // publish date so the history interleaves into the river chronologically
    // instead of clumping on top as one just-arrived batch.
    if feed.last_fetched_at.is_none() {
        for form in &mut forms {
            form.created_at = form.published_at;
        }
    }

    let new_posts = Post::create_many(pool, forms).await?;
    info!("Got {} new posts for feed {}", new_posts.len(), feed.title);

    Feed::update(
        pool,
        FeedUpdateForm {
            id: feed.id,
            title: None,
            url: None,
            ttl: None,
            tag: None,
            tag_color: None,
            filters: None,
            last_fetched_at: Some(Utc::now()),
        },
    )
    .await?;

    if new_posts.is_empty() {
        return Ok(());
    }

    for webhook in webhooks {
        // A broken custom template must not abort the poll or send garbage:
        // log and skip this delivery.
        match wyrm_webhook::get_payload(webhook, feed, &new_posts) {
            Ok(payload) => {
                let http = http.clone();
                let webhook = webhook.clone();
                tokio::spawn(async move {
                    if let Err(e) = http.post_json(&webhook.url, &payload).await {
                        warn!("webhook {} delivery error: {e}", webhook.name);
                    }
                });
            }
            Err(e) => error!("skipping webhook for feed {}: {e}", feed.title),
        }
    }

    Ok(())
}
