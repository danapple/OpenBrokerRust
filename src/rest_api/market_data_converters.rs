use crate::exchange_interface;
use crate::instrument_manager::InstrumentManager;
use crate::rest_api::market_data::{LastTrade, MarketDepth, PriceLevel};

impl exchange_interface::market_data::LastTrade {
    pub fn to_rest_api_last_trade(&self, instrument_manager: &InstrumentManager) -> LastTrade {
        LastTrade {
            version_number: self.sequence_number,
            instrument_key: instrument_manager.get_instrument_by_exchange_instrument_id(self.instrument_id).unwrap().unwrap().instrument_key,
            create_time: self.create_time,
            price: self.price,
            quantity: self.quantity,
        }
    }
}

impl exchange_interface::market_data::MarketDepth {
    pub fn to_rest_api_position(&self, instrument_manager: &InstrumentManager) -> MarketDepth {
        let buys = self.buys.iter().map(|buy| { buy.to_rest_api_price_level() } ).collect();
        let sells = self.sells.iter().map(|sell| { sell.to_rest_api_price_level() } ).collect();

        MarketDepth {
            version_number: self.sequence_number,
            instrument_key: instrument_manager.get_instrument_by_exchange_instrument_id(self.instrument_id).unwrap().unwrap().instrument_key,
            create_time: self.create_time,
            buys,
            sells,
        }
    }
}

impl exchange_interface::market_data::PriceLevel {
    pub fn to_rest_api_price_level(&self) -> PriceLevel {
        PriceLevel {
            price: self.price,
            quantity: self.quantity,
        }
    }
}
