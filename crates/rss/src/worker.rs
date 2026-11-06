use crate::{
    filter::CompiledFilters,
    http::{FeedResponse, HttpClient, HttpConfig},
};
use chrono::Utc;
use database::{
    DatabasePool,
    models::{
        feed::Feed,
        feed_poll_state::FeedPollState,
        host_credential::HostCredential,
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
    pub runtime_settings: Arc<RwLock<RuntimeSettings>>,
}

impl FeedWorker {
    pub fn new(pool: DatabasePool, runtime_settings: Arc<RwLock<RuntimeSettings>>) -> Self {
        Self {
            db_pool: pool,
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

        let mut poll_states = FeedPollState::all_by_feed(&self.db_pool).await?;

        let due_feeds: Vec<Feed> = feeds
            .into_iter()
            .filter(|f| f.is_due() && !f.is_paused)
            .collect();

        info!("Processing {} due feeds", due_feeds.len());

        let host_credentials = HostCredential::get_all(&self.db_pool).await?;

        // Build the http client from in-memory runtime settings
        // as settings may have been updated.
        let http = HttpClient::new(
            &HttpConfig::from(&*self.runtime_settings.read()?),
            host_credentials,
        )?;

        let feed_webhooks = views::webhook::all_by_feed(&self.db_pool).await?;
        let feed_folders = views::folder::all_by_feed(&self.db_pool).await?;

        // Each in-flight feed holds ~1 pool connection. Reserve headroom so polling
        // can never starve the API handlers of the shared pool.
        let permits = (self.db_pool.status().max_size / 2).max(1);
        let semaphore = Arc::new(Semaphore::new(permits));

        let tasks: Vec<_> = due_feeds
            .into_iter()
            .map(|feed| {
                let poll_state = poll_states
                    .remove(&feed.id)
                    .filter(|state| state.matches(&feed));
                let http = http.clone();
                let pool = self.db_pool.clone();
                let webhooks = feed_webhooks.get(&feed.id).cloned().unwrap_or_default();
                let folder = feed_folders.get(&feed.id).map(|f| f.name.clone());
                let semaphore = semaphore.clone();
                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let task = FeedTask {
                        feed,
                        poll_state,
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
    poll_state: Option<FeedPollState>,
    /// List of webhooks attached to the feed
    webhooks: Vec<Webhook>,
    /// Resolved folder name for webhook payloads; `None` = standalone feed.
    folder: Option<String>,
}

/// Fetch -> Parse -> Store -> Update -> Notify
async fn process_feed(pool: &DatabasePool, http: &HttpClient, task: &FeedTask) -> WyrmResult<()> {
    info!("Processing feed {}", task.feed.title);

    let mut state = task.poll_state.clone().unwrap_or_else(|| FeedPollState {
        feed_id: task.feed.id,
        source_url: task.feed.url.clone(),
        filter_signature: FeedPollState::filters_key(&task.feed),
        checked_at: Utc::now(),
        last_error: None,
        etag: None,
        last_modified: None,
    });
    let result = fetch_and_store(pool, http, task, &mut state).await;
    state.checked_at = Utc::now();
    state.last_error = result.as_ref().err().map(poll_error_message);
    state.record(pool, result.is_ok()).await?;
    result
}

/// Deliberately avoid persisting errors containing URLs, credentials or SQL details.
fn poll_error_message(error: &wyrm_utils::error::WyrmError) -> String {
    use wyrm_utils::error::{HttpClientError, WyrmError};
    match error {
        WyrmError::HttpClient(HttpClientError::Unauthorized) => {
            "Authentication failed (HTTP 401/403)".into()
        }
        WyrmError::HttpClient(HttpClientError::ClientError(error)) if error.is_timeout() => {
            "Request timed out".into()
        }
        WyrmError::HttpClient(HttpClientError::ClientError(error)) => error
            .status()
            .map(|status| format!("HTTP {status}"))
            .unwrap_or_else(|| "Could not connect to the feed server".into()),
        WyrmError::HttpClient(HttpClientError::ResponseTooLarge(_)) => {
            "Feed exceeds the download size limit".into()
        }
        WyrmError::ParseFeedError(_) => "Response is not a valid RSS or Atom feed".into(),
        WyrmError::Database(_) => "Could not save feed updates".into(),
        _ => "Could not refresh this feed; check server logs".into(),
    }
}

async fn fetch_and_store(
    pool: &DatabasePool,
    http: &HttpClient,
    task: &FeedTask,
    state: &mut FeedPollState,
) -> WyrmResult<()> {
    let refresh_icon = (state.etag.is_some() || state.last_modified.is_some())
        && crate::icon::needs_feed_icon(pool, task.feed.id).await;
    let response = http
        .fetch_feed(
            &task.feed.url,
            if refresh_icon {
                None
            } else {
                state.etag.as_deref()
            },
            if refresh_icon {
                None
            } else {
                state.last_modified.as_deref()
            },
        )
        .await?;
    let (bytes, etag, last_modified) = match response {
        FeedResponse::NotModified {
            etag,
            last_modified,
        } => {
            state.etag = etag;
            state.last_modified = last_modified;
            return Ok(());
        }
        FeedResponse::Modified {
            body,
            etag,
            last_modified,
            ..
        } => (body, etag, last_modified),
    };
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

    // Commit validators only after parsing and storage succeed. A bad 200 response
    // must not make the next request skip a body we never successfully processed.
    state.etag = etag;
    state.last_modified = last_modified;

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

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::test_server::{response, serve};
    use database::{
        create_pool,
        establish_sync_connection,
        models::{
            feed::FeedInsertForm,
            feed_icon::{FeedIcon, FeedIconInsertForm},
        },
        run_migrations,
        views::post::PostQuery,
    };
    use wyrm_utils::config::WyrmStartupConfig;

    struct TestDb {
        pool: DatabasePool,
        path: String,
    }
    impl TestDb {
        async fn new() -> Self {
            let path = std::env::temp_dir()
                .join(format!(
                    "wyrm_poll_{}.db",
                    Utc::now().timestamp_nanos_opt().unwrap()
                ))
                .to_string_lossy()
                .into_owned();
            let config = WyrmStartupConfig {
                database_connection: path.clone(),
                database_pool_size: 2,
                ..Default::default()
            };
            let mut conn = establish_sync_connection(&config).unwrap();
            run_migrations(&mut conn).unwrap();
            drop(conn);
            Self {
                pool: create_pool(&config).await.unwrap(),
                path,
            }
        }
    }
    impl Drop for TestDb {
        fn drop(&mut self) {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{suffix}", self.path));
            }
        }
    }

    async fn poll(
        pool: &DatabasePool,
        http: &HttpClient,
        id: database::newtypes::FeedId,
    ) -> WyrmResult<()> {
        let feed = Feed::get(pool, id).await.unwrap();
        let poll_state = FeedPollState::all_by_feed(pool)
            .await
            .unwrap()
            .remove(&id)
            .filter(|s| s.matches(&feed));
        process_feed(
            pool,
            http,
            &FeedTask {
                feed,
                poll_state,
                webhooks: vec![],
                folder: None,
            },
        )
        .await
    }

    #[tokio::test]
    async fn polling_revalidates_recovers_and_does_not_cache_invalid_bodies() {
        let rss = r#"<rss version="2.0"><channel><title>Fixture</title><link>https://example.com</link><description>test</description><item><title>Post</title><link>https://example.com/post</link><guid>post-1</guid></item></channel></rss>"#;
        let (url, server) = serve(vec![
            response("200 OK", "ETag: \"v1\"\r\n", rss),
            response("304 Not Modified", "", ""),
            response("503 Service Unavailable", "", "down"),
            response("200 OK", "ETag: \"bad\"\r\n", "not a feed"),
            response("304 Not Modified", "", ""),
            response("200 OK", "", rss),
            response("200 OK", "ETag: \"v3\"\r\n", rss),
            response("200 OK", "", rss),
        ])
        .await;
        let db = TestDb::new().await;
        let feed = Feed::create(
            &db.pool,
            FeedInsertForm {
                title: "Fixture".into(),
                url,
                ttl: 60,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(crate::icon::needs_feed_icon(&db.pool, feed.id).await);
        // Avoid icon requests: this fixture exclusively exercises feed polling.
        FeedIcon::create(
            &db.pool,
            FeedIconInsertForm {
                feed_id: feed.id,
                data: Some(vec![1]),
                content_type: Some("image/png".into()),
            },
        )
        .await
        .unwrap();
        let http = HttpClient::new(
            &HttpConfig {
                timeout: 2,
                connect_timeout: 2,
                retries: 0,
                user_agent: None,
            },
            vec![],
        )
        .unwrap();
        poll(&db.pool, &http, feed.id).await.unwrap();
        let first = Feed::get(&db.pool, feed.id)
            .await
            .unwrap()
            .last_fetched_at
            .unwrap();
        poll(&db.pool, &http, feed.id).await.unwrap();
        let unchanged = Feed::get(&db.pool, feed.id)
            .await
            .unwrap()
            .last_fetched_at
            .unwrap();
        assert!(unchanged >= first);
        assert!(poll(&db.pool, &http, feed.id).await.is_err());
        let states = FeedPollState::all_by_feed(&db.pool).await.unwrap();
        assert!(
            states[&feed.id]
                .last_error
                .as_ref()
                .unwrap()
                .contains("503")
        );
        assert_eq!(
            Feed::get(&db.pool, feed.id).await.unwrap().last_fetched_at,
            Some(unchanged)
        );
        assert!(poll(&db.pool, &http, feed.id).await.is_err());
        let states = FeedPollState::all_by_feed(&db.pool).await.unwrap();
        assert_eq!(states[&feed.id].etag.as_deref(), Some("\"v1\""));
        assert_eq!(
            states[&feed.id].last_error.as_deref(),
            Some("Response is not a valid RSS or Atom feed")
        );
        poll(&db.pool, &http, feed.id).await.unwrap();
        assert!(
            FeedPollState::all_by_feed(&db.pool).await.unwrap()[&feed.id]
                .last_error
                .is_none()
        );
        poll(&db.pool, &http, feed.id).await.unwrap();
        assert!(
            FeedPollState::all_by_feed(&db.pool).await.unwrap()[&feed.id]
                .etag
                .is_none()
        );
        poll(&db.pool, &http, feed.id).await.unwrap();
        let current = Feed::get(&db.pool, feed.id).await.unwrap();
        Feed::update(
            &db.pool,
            database::models::feed::FeedUpdateForm {
                id: feed.id,
                title: None,
                url: None,
                ttl: None,
                folder: None,
                filters: Some(vec![Some("sponsored".into())].into()),
                is_paused: None,
                last_fetched_at: None,
                display_mode: None,
            },
        )
        .await
        .unwrap();
        assert!(FeedPollState::all_by_feed(&db.pool).await.unwrap()[&feed.id].matches(&current));
        poll(&db.pool, &http, feed.id).await.unwrap();
        let posts = PostQuery {
            feed_id: Some(feed.id),
            ..Default::default()
        }
        .list(&db.pool, 100)
        .await
        .unwrap();
        assert_eq!(
            posts.items.len(),
            1,
            "304/failures/repeated bodies must not duplicate posts"
        );
        let requests = server.await.unwrap();
        assert!(!requests[0].to_lowercase().contains("if-none-match"));
        for request in &requests[1..6] {
            assert!(request.contains("\"v1\""));
        }
        assert!(!requests[6].to_lowercase().contains("if-none-match"));
        assert!(
            !requests[7].to_lowercase().contains("if-none-match"),
            "filter edits bypass stored validators"
        );
        assert!(!crate::icon::needs_feed_icon(&db.pool, feed.id).await);
        FeedIcon::delete(&db.pool, feed.id).await.unwrap();
        assert!(crate::icon::needs_feed_icon(&db.pool, feed.id).await);
    }

    #[test]
    fn persisted_errors_do_not_expose_sensitive_details() {
        use wyrm_utils::error::WyrmError;
        let error = WyrmError::WorkerError("https://user:secret@example.com?token=private".into());
        let message = poll_error_message(&error);
        assert!(!message.contains("secret") && !message.contains("private"));
    }
}
