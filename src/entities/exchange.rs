use crate::exchange_interface;
use crate::rest_api::exchange::{AssetClass, InstrumentStatus};
use crate::time::current_time_millis;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Instrument {
    pub instrument_id: i64,
    pub exchange_id: i32,
    pub exchange_instrument_id: i64,
    pub status: InstrumentStatus,
    pub symbol: String,
    pub asset_class: AssetClass,
    pub description: String,
    pub expiration_time: i64
}

pub fn instrument_from_exchange_instrument(exchange_instrument: &exchange_interface::trading::Instrument, exchange_id: i32) -> Instrument {
    Instrument {
        instrument_id: 0,
        exchange_id,
        exchange_instrument_id: exchange_instrument.instrument_id,
        status: InstrumentStatus::Active,
        symbol: format!("Symbol:{}", exchange_instrument.instrument_id),
        asset_class: AssetClass::Equity,
        description: format!("Description:{}", exchange_instrument.instrument_id),
        expiration_time: current_time_millis() + 365 * 86400 * 1000,
    }
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
