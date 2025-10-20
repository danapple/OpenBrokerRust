use confik::Configuration;
use serde::Deserialize;

#[derive(Debug, Default, Configuration, Clone)]
pub struct BrokerConfig {
    pub server_addr: String,
    pub websocket_addr: String,
    pub exchange_url: String,
    pub exchange_websocket_address: String,
    pub exchange_websocket_virtual_host: String,
    pub broker_key: String,
    pub log_level: String,  
    #[confik(from = DbConfig)]
    pub pg: deadpool_postgres::Config,
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