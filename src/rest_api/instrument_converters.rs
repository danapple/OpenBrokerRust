use crate::entities::exchange::Exchange;
use crate::{entities, rest_api};

impl entities::exchange::Instrument {
    pub fn to_rest_api_instrument(&self, exchange: &Exchange) -> rest_api::exchange::Instrument {
        rest_api::exchange::Instrument {
            instrument_id: self.instrument_id,
            status: self.status.clone(),
            symbol: self.symbol.clone(),
            asset_class: self.asset_class.clone(),
            exchange_code: exchange.code.clone(),
            description: self.description.clone(),
            expiration_time: self.expiration_time,
        }
    }
}