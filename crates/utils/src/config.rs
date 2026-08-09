use crate::result::WyrmResult;
use config::{Config, Environment, File};
use serde::Deserialize;
use smart_default::SmartDefault;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone, Deserialize, SmartDefault)]
#[serde(default)]
/// Built from wyrm.toml or env vars
pub struct WyrmStartupConfig {
    /// Database connection URI
    #[default("postgres://wyrm:wyrm@localhost/wyrm")]
    pub database_connection: String,
    /// Max number of active database connections
    #[default(30)]
    pub database_pool_size: usize,
    /// Bind to IP addr
    #[default(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
    pub bind: IpAddr,
    /// Port to listen on
    #[default(3001)]
    pub port: u16,
    /// API key
    pub api_key: Option<String>,
}

impl WyrmStartupConfig {
    pub fn load() -> WyrmResult<Self> {
        Config::builder()
            .add_source(File::with_name("config/wyrm.toml").required(false))
            .add_source(Environment::with_prefix("WYRM"))
            .build()?
            .try_deserialize()
            .map_err(Into::into)
    }
}
