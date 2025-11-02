use crate::entities::exchange::Exchange;
use crate::{dtos, entities};

impl entities::exchange::Instrument {
    pub fn to_rest_api_instrument(&self, exchange: &Exchange) -> dtos::exchange::Instrument {
        dtos::exchange::Instrument {
            instrument_key: self.instrument_key.clone(),
            status: self.status.clone(),
            symbol: self.symbol.clone(),
            asset_class: self.asset_class.clone(),
            exchange_code: exchange.code.clone(),
            description: self.description.clone(),
            expiration_time: self.expiration_time,
        }
    }
}

impl dtos::exchange::Exchange {
    pub fn to_entities_exchange(&self) -> entities::exchange::Exchange {
        Exchange {
            exchange_id: 0,
            code: self.code.clone(),
            url: self.url.clone(),
            websocket_url: self.websocket_url.clone(),
            description: self.description.clone(),
            api_key: self.api_key.clone(),
        }
    }
}