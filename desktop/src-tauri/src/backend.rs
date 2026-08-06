use crate::{
    error::{WyrmDesktopError, WyrmDesktopResult},
    youtube::youtube_embed,
};
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::from_fn, web};
use api_utils::context::WyrmContext;
use database::{models::settings::Settings, utils::settings::RuntimeSettings};
use rand::{RngExt, distr::Alphanumeric};
use std::{
    net::{IpAddr, Ipv4Addr, TcpListener},
    path::PathBuf,
    sync::{Arc, OnceLock, RwLock},
};
use tracing::info;
use wyrm_rss::{
    http::{HttpClient, HttpConfig},
    worker::{FeedWorker, WorkerCommand},
};
use wyrm_utils::{
    config::WyrmStartupConfig,
    error::{DatabaseError, HttpServerError},
    middleware::api_key_middleware,
};

/// Info the frontend needs to talk to the embedded backend, resolved once at
/// startup since the port is OS-assigned and the api key is generated fresh
/// per launch.
#[derive(Clone, serde::Serialize)]
pub struct ServerInfo {
    pub base_url: String,
    pub api_key: String,
}

static SERVER_INFO: OnceLock<ServerInfo> = OnceLock::new();

#[tauri::command]
pub fn server_info() -> Option<ServerInfo> {
    SERVER_INFO.get().cloned()
}

/// Nothing to tear down: SQLite has no server process, and the pool's
/// connections flush and close on drop. Kept as a no-op so the exit paths in
/// `lib.rs` (window close, Ctrl+C) stay unchanged, and so there's somewhere
/// obvious to hang a WAL checkpoint if that ever proves necessary.
pub async fn shutdown() {}

/// Database filename inside the app-data directory. WAL mode puts `-wal` and
/// `-shm` sidecars next to it.
const DATABASE_FILE: &str = "wyrm.db";

pub async fn start_backend(app: &tauri::AppHandle) -> WyrmDesktopResult<()> {
    use tauri::{Emitter, Manager};

    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| WyrmDesktopError::AppDataDir(e.to_string()))?;
    let info = bootstrap(data_dir).await?;
    let _ = app.emit("wyrm://backend-ready", ());
    let _ = SERVER_INFO.set(info);
    Ok(())
}

/// Opens the SQLite database in the given app-data directory, migrates it and
/// starts the actix backend, returning how to reach it. Kept independent of
/// `AppHandle` so it can be exercised directly (e.g. from tests) without a
/// GUI/display.
pub async fn bootstrap(data_dir: PathBuf) -> WyrmDesktopResult<ServerInfo> {
    std::fs::create_dir_all(&data_dir)?;

    let api_key: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    // For the sqlite backend `database_connection` is a file path, not a URL.
    // SQLite creates the file on first open, so there's nothing to provision.
    let db_path = data_dir.join(DATABASE_FILE);
    info!("Using database at {}", db_path.display());

    let startup_conf = WyrmStartupConfig {
        database_connection: db_path.to_string_lossy().into_owned(),
        api_key: Some(api_key.clone()),
        bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
        ..Default::default()
    };

    info!("Establishing database connection for migrations");
    let mut db_sync_conn = database::establish_sync_connection(&startup_conf)?;

    info!("Running database migrations");
    database::run_migrations(&mut db_sync_conn)
        .map_err(|e| wyrm_utils::error::WyrmError::from(DatabaseError::MigrationError(e)))?;

    info!("Creating database pool");
    let db_pool = database::create_pool(&startup_conf).await?;

    let settings = Settings::get(&db_pool).await?;
    let rs = RuntimeSettings::from(&settings);

    info!("Building http client");
    let http = HttpClient::new(&HttpConfig::from(&rs))?;

    let runtime_settings = Arc::new(RwLock::new(rs));
    let (tx, rx) = tokio::sync::mpsc::channel::<WorkerCommand>(1);

    let ctx = web::Data::new(WyrmContext {
        db_pool: db_pool.clone(),
        runtime_settings: runtime_settings.clone(),
        http: http.clone(),
        worker_tx: tx,
    });

    tokio::spawn(async move {
        let mut rss_worker = FeedWorker::new(db_pool, http, runtime_settings);
        let _ = rss_worker.run(rx).await;
    });

    info!("Starting up embedded HTTP server");
    let listener = TcpListener::bind((startup_conf.bind, 0))?;
    let port = listener.local_addr()?.port();

    let api_key_data = startup_conf.api_key.clone();
    let server = HttpServer::new(move || {
        App::new()
            .app_data(ctx.clone())
            .app_data(web::Data::new(api_key_data.clone()))
            // The webview's origin (tauri://localhost, http://tauri.localhost,
            // etc.) differs from this server's, so cross-origin fetches need
            // CORS. Permissive is acceptable here: the server only binds
            // 127.0.0.1 and every request still needs the per-launch api key.
            .wrap(Cors::permissive())
            .route("/youtube-embed", web::get().to(youtube_embed))
            .service(
                web::scope("")
                    .wrap(from_fn(api_key_middleware))
                    .configure(api_routes::config),
            )
    })
    .listen(listener)
    .map_err(HttpServerError::BindError)
    .map_err(wyrm_utils::error::WyrmError::from)?
    .run();

    tokio::spawn(server);

    Ok(ServerInfo {
        base_url: format!("http://127.0.0.1:{port}"),
        api_key,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Proves the desktop bootstrap end-to-end: create a SQLite database in a
    /// scratch dir, run migrations, start the actix server on an OS-assigned
    /// port, and hit a real API route through it.
    #[tokio::test(flavor = "multi_thread")]
    async fn bootstrap_serves_api_requests() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
        let data_dir =
            std::env::temp_dir().join(format!("wyrm-desktop-poc-{}", std::process::id()));

        let info = bootstrap(data_dir.clone())
            .await
            .expect("bootstrap should succeed");

        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/api/v1/settings", info.base_url))
            .header("x-api-key", &info.api_key)
            .send()
            .await
            .expect("request should succeed");

        assert!(
            res.status().is_success(),
            "unexpected status: {}",
            res.status()
        );

        let _ = std::fs::remove_dir_all(&data_dir);
    }
}
