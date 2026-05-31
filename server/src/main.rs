mod server;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use wyrm_utils::result::WyrmResult;

#[tokio::main]
async fn main() -> WyrmResult<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt().with_env_filter(filter).init();

    server::start_server().await?;

    Ok(())
}
