use actix_web::{App, HttpServer, dev::ServerHandle, middleware::from_fn, web};
use api_utils::context::WyrmContext;
use database::{models::settings::Settings, utils::settings::RuntimeSettings};
use std::sync::{Arc, RwLock};
use tracing::info;
use wyrm_rss::{
    http::{HttpClient, HttpConfig},
    worker::{FeedWorker, WorkerCommand},
};
use wyrm_utils::{
    config::WyrmStartupConfig,
    error::{DatabaseError, HttpServerError},
    middleware::api_key_middleware,
    result::WyrmResult,
};

pub async fn start_server() -> WyrmResult<()> {
    info!("Loading settings");
    let startup_conf = WyrmStartupConfig::load();

    info!("Establishing database connection for migrations");
    let mut db_sync_conn = database::establish_sync_connection(&startup_conf)?;

    info!("Running database migrations");
    database::run_migrations(&mut db_sync_conn).map_err(DatabaseError::MigrationError)?;

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

    info!("Starting up HTTP server");
    let server_handle = spin_up_http_server(startup_conf, ctx)?;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => server_handle.stop(true).await
    }

    Ok(())
}

fn spin_up_http_server(
    startup_conf: WyrmStartupConfig,
    ctx: web::Data<WyrmContext>,
) -> WyrmResult<ServerHandle> {
    let bind = startup_conf.bind;
    let port = startup_conf.port;
    let server = HttpServer::new(move || {
        App::new()
            .app_data(ctx.clone())
            .app_data(web::Data::new(startup_conf.api_key.clone()))
            .wrap(from_fn(api_key_middleware))
            .configure(api_routes::config)
    })
    .bind((bind, port))
    .map_err(HttpServerError::BindError)?
    .run();

    let h = server.handle();
    tokio::spawn(server);

    Ok(h)
}
