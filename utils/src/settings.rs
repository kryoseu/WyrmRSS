use std::net::{IpAddr, Ipv4Addr};

use config::{Config, Environment, File};
use serde::Deserialize;
use smart_default::SmartDefault;

#[derive(Debug, Clone, Deserialize, SmartDefault)]
#[serde(default)]
pub struct DatabaseConfig {
    /// Database URI
    #[default("postgres://wyrm:wyrm@localhost/wyrm")]
    pub connection: String,
    /// Max number of active database connections
    #[default(30)]
    pub pool_size: usize,
}

#[derive(Debug, Clone, Deserialize, SmartDefault)]
#[serde(default)]
pub struct HttpConfig {
    /// Response timeout
    #[default(30)]
    pub timeout: u64,
    //// Conn timeout
    #[default(30)]
    pub connect_timeout: u64,
    /// Max retries
    #[default(3)]
    pub retries: u32,
    #[default(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")).to_string())]
    pub user_agent: String,
}

#[derive(Debug, Clone, Deserialize, SmartDefault)]
#[serde(default)]
pub struct FeedConfig {
    /// Pagination size
    #[default(100)]
    pub page_size: i64,
}

#[derive(Debug, Clone, Deserialize, SmartDefault)]
#[serde(default)]
pub struct WyrmSettings {
    /// Feed settings
    pub feed: FeedConfig,
    /// Database settings
    pub database: DatabaseConfig,
    /// Http settings
    pub http: HttpConfig,
    /// Bind to IP addr
    #[default(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
    pub bind: IpAddr,
    /// Port to listen on
    #[default(3001)]
    pub port: u16,
}

impl WyrmSettings {
    pub fn load() -> Self {
        Config::builder()
            .add_source(File::with_name("config/wyrm.toml").required(false))
            .add_source(Environment::with_prefix("WYRM").separator("_"))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}
