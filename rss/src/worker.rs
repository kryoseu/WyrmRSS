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
        self.expire_posts().await?;

        let feeds = Feed::get_all(&self.db_pool).await?;

        let due_feeds: Vec<Feed> = feeds
            .into_iter()
            .filter(|f| f.is_due() && !f.is_paused)
            .collect();

        info!("Processing {} due feeds", due_feeds.len());

        // Re-configure the http client from in-memory runtime settings
        // as settings may have been updated.
        self.http = HttpClient::new(&HttpConfig::from(&*self.runtime_settings.read()?))?;

        let feed_webhooks = views::webhook::all_by_feed(&self.db_pool).await?;
        let feed_folders = views::folder::all_by_feed(&self.db_pool).await?;

        // Each in-flight feed holds ~1 pool connection. Reserve headroom so polling
        // can never starve the API handlers of the shared pool.
        let permits = (self.db_pool.status().max_size / 2).max(1);
        let semaphore = Arc::new(Semaphore::new(permits));

        let tasks: Vec<_> = due_feeds
            .into_iter()
            .map(|feed| {
                let http = self.http.clone();
                let pool = self.db_pool.clone();
                let webhooks = feed_webhooks.get(&feed.id).cloned().unwrap_or_default();
                let folder = feed_folders.get(&feed.id).map(|f| f.name.clone());
                let semaphore = semaphore.clone();
                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let task = FeedTask {
                        feed,
                        webhooks,
                        folder,
                    };
                    if let Err(e) = process_feed(&pool, &http, &task).await {
                        error!("Failed to process feed {}: {e}", task.feed.url);
                    }
                })
            })
            .collect();

        join_all(tasks).await;

        Ok(())
    }

    /// Deletes read/unread posts older than the configured thresholds.
    /// Bookmarked posts are never deleted. Failures are logged rather
    /// than propagated so expiry can never block feed polling.
    async fn expire_posts(&self) -> WyrmResult<()> {
        let (expire_read_after_days, expire_unread_after_days) = {
            let settings = self.runtime_settings.read()?;
            (
                settings.expire_read_after_days,
                settings.expire_unread_after_days,
            )
        };
        if let Some(days) = expire_read_after_days {
            match Post::expire_read(&self.db_pool, days).await {
                Ok(0) => {}
                Ok(n) => info!("Expired {n} read posts"),
                Err(e) => error!("failed to expire read posts: {e}"),
            }
        }
        if let Some(days) = expire_unread_after_days {
            match Post::expire_unread(&self.db_pool, days).await {
                Ok(0) => {}
                Ok(n) => info!("Expired {n} unread posts"),
                Err(e) => error!("failed to expire unread posts: {e}"),
            }
        }
        Ok(())
    }
}

/// Per-feed data for one poll task
pub struct FeedTask {
    /// The feed
    pub feed: Feed,
    /// List of webhooks attached to the feed
    webhooks: Vec<Webhook>,
    /// Resolved folder name for webhook payloads; `None` = standalone feed.
    folder: Option<String>,
}

/// Fetch -> Parse -> Store -> Update -> Notify
async fn process_feed(pool: &DatabasePool, http: &HttpClient, task: &FeedTask) -> WyrmResult<()> {
    info!("Processing feed {}", task.feed.title);

    let (bytes, _) = http.fetch(&task.feed.url).await?;

    let parsed = feed_rs::parser::parse(&bytes[..])?;

    // Best-effort: resolves the icon for feeds that never got one (OPML
    // imports, pre-icon feeds) and retries week-old misses.
    crate::icon::ensure_feed_icon(pool, http, &task.feed, &parsed).await;

    let filters = CompiledFilters::new(&task.feed.filters);

    let mut forms: Vec<PostInsertForm> = parsed
        .entries
        .into_iter()
        .filter(|entry| !filters.excludes(entry))
        .map(|entry| PostInsertForm::from_entry(entry, task.feed.id))
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
    if task.feed.last_fetched_at.is_none() {
        for form in &mut forms {
            form.created_at = form.published_at;
        }
    }

    let new_posts = Post::create_many(pool, forms).await?;
    info!(
        "Got {} new posts for feed {}",
        new_posts.len(),
        task.feed.title
    );

    Feed::update(
        pool,
        FeedUpdateForm {
            id: task.feed.id,
            title: None,
            url: None,
            ttl: None,
            folder: None,
            filters: None,
            is_paused: None,
            last_fetched_at: Some(Utc::now()),
        },
    )
    .await?;

    if new_posts.is_empty() {
        return Ok(());
    }

    for webhook in &task.webhooks {
        // A broken custom template must not abort the poll or send garbage:
        // log and skip this delivery.
        match wyrm_webhook::get_payload(webhook, &task.feed, task.folder.as_deref(), &new_posts) {
            Ok(payload) => {
                let http = http.clone();
                let webhook = webhook.clone();
                tokio::spawn(async move {
                    if let Err(e) = http.post_json(&webhook.url, &payload).await {
                        warn!("webhook {} delivery error: {e}", webhook.name);
                    }
                });
            }
            Err(e) => error!("skipping webhook for feed {}: {e}", task.feed.title),
        }
    }

    Ok(())
}
