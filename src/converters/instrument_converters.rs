use crate::dtos::exchange::{AssetClass, InstrumentStatus};
use crate::entities::exchange::Instrument;
use crate::exchange_interface;
use uuid::Uuid;

impl exchange_interface::instrument::Instrument {
    pub fn to_entities_instrument(&self, 
                                  exchange_id: i32) -> Instrument {
        Instrument {
            instrument_id: 0,
            instrument_key: Uuid::new_v4().simple().to_string(),
            exchange_id,
            exchange_instrument_id: self.instrument_id,
            status: exchange_instrument_status_to_entities_instrument_status(&self.status),
            symbol: self.symbol.clone(),
            asset_class: exchange_asset_class_to_entities_asset_class(&self.asset_class),
            description: self.description.clone(),
            expiration_time: self.expiration_time,
        }
    }
}

fn exchange_asset_class_to_entities_asset_class(asset_class: &exchange_interface::instrument::AssetClass)
                                                    -> AssetClass {
    match asset_class {
        exchange_interface::instrument::AssetClass::Equity => AssetClass::Equity,
        exchange_interface::instrument::AssetClass::Option => AssetClass::Option,
        exchange_interface::instrument::AssetClass::Future => AssetClass::Future,
        exchange_interface::instrument::AssetClass::Commodity => AssetClass::Commodity,
    }
}

fn exchange_instrument_status_to_entities_instrument_status(instrument_status: &exchange_interface::instrument::InstrumentStatus)
                                                                -> InstrumentStatus {
    match instrument_status {
        exchange_interface::instrument::InstrumentStatus::Active => InstrumentStatus::Active,
        exchange_interface::instrument::InstrumentStatus::Inactive => InstrumentStatus::Inactive,
    }
}