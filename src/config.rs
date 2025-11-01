use confik::Configuration;
use serde::Deserialize;

#[derive(Debug, Default, Configuration, Clone)]
pub struct BrokerConfig {
    pub server_addr: String,
    pub log_level: String,
    #[confik(from = DbConfig)]
    pub pg: deadpool_postgres::Config,
    pub redis_addr: String,
    pub password_key: String,
    pub session_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct DbConfig(deadpool_postgres::Config);

impl From<DbConfig> for deadpool_postgres::Config {
    fn from(value: DbConfig) -> Self {
        value.0
    }
}

impl confik::Configuration for DbConfig {
    type Builder = Option<Self>;
}