use crate::exchange_interface;
use crate::rest_api::exchange::{AssetClass, InstrumentStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

pub fn instrument_from_exchange_instrument(exchange_instrument: &exchange_interface::instrument::Instrument, exchange_id: i32) -> Instrument {
    Instrument {
        instrument_id: 0,
        instrument_key: Uuid::new_v4().simple().to_string(),
        exchange_id,
        exchange_instrument_id: exchange_instrument.instrument_id,
        status: exchange_instrument_status_to_entities_instrument_status(&exchange_instrument.status),
        symbol: exchange_instrument.symbol.clone(),
        asset_class: exchange_asset_class_to_entities_asset_class(&exchange_instrument.asset_class),
        description: exchange_instrument.description.clone(),
        expiration_time: exchange_instrument.expiration_time,
    }
}

pub fn exchange_asset_class_to_entities_asset_class(asset_class: &exchange_interface::instrument::AssetClass)
                                                    -> AssetClass {
    match asset_class {
        exchange_interface::instrument::AssetClass::Equity => AssetClass::Equity,
        exchange_interface::instrument::AssetClass::Option => AssetClass::Option,
        exchange_interface::instrument::AssetClass::Future => AssetClass::Future,
        exchange_interface::instrument::AssetClass::Commodity => AssetClass::Commodity,
    }
}

pub fn exchange_instrument_status_to_entities_instrument_status(instrument_status: &exchange_interface::instrument::InstrumentStatus)
                                           -> InstrumentStatus {
    match instrument_status {
        exchange_interface::instrument::InstrumentStatus::Active => InstrumentStatus::Active,
        exchange_interface::instrument::InstrumentStatus::Inactive => InstrumentStatus::Inactive,
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
