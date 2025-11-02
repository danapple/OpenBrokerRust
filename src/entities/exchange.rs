use crate::dtos::exchange::{AssetClass, InstrumentStatus};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Instrument {
    pub instrument_id: i64,
    pub instrument_key: String,
    pub exchange_id: i32,
    pub exchange_instrument_id: i64,
    pub status: InstrumentStatus,
    pub symbol: String,
    pub asset_class: AssetClass,
    pub description: String,
    pub expiration_time: i64
}


#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Exchange {
    pub exchange_id: i32,
    pub code: String,
    pub url: String,
    pub websocket_url: String,
    pub description: String,
    pub api_key: String,
}
