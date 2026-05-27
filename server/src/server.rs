use actix_web::dev::ServerHandle;
use actix_web::{App, HttpServer, web};
use api_utils::context::WyrmContext;
use tracing::info;
use utils::error::{DatabaseError, HttpServerError};
use utils::result::WyrmResult;
use utils::settings::WyrmSettings;
use wyrm_rss::http::HttpClient;
use wyrm_rss::worker::FeedWorker;

pub async fn start_server() -> WyrmResult<()> {
    info!("Loading settings");
    let settings = WyrmSettings::load();

    info!("Establishing database connection for migrations");
    let mut db_sync_conn = database::establish_sync_connection(&settings)?;

    info!("Running database migrations");
    database::run_migrations(&mut db_sync_conn).map_err(DatabaseError::MigrationError)?;

    info!("Creating database pool");
    let db_pool = database::create_pool(&settings).await?;

    info!("Building http client");
    let http = HttpClient::builder(&settings).build()?;

    let ctx = web::Data::new(WyrmContext {
        db_pool: db_pool.clone(),
        settings: settings.clone(),
        http: http.clone(),
    });

    tokio::spawn(async move {
        let mut rss_worker = FeedWorker::new(db_pool, http);
        rss_worker.run().await;
    });

    info!("Starting up HTTP server");
    let server_handle = spin_up_http_server(ctx)?;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => server_handle.stop(true).await
    }

    Ok(())
}

fn spin_up_http_server(ctx: web::Data<WyrmContext>) -> WyrmResult<ServerHandle> {
    let bind = ctx.settings.bind;
    let port = ctx.settings.port;

    let server = HttpServer::new(move || {
        App::new()
            .app_data(ctx.clone())
            .configure(api_routes::config)
    })
    .bind((bind, port))
    .map_err(HttpServerError::BindError)?
    .run();

    let h = server.handle();
    tokio::spawn(server);

    Ok(h)
}
