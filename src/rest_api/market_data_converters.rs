use crate::exchange_interface;
use crate::rest_api::market_data::{LastTrade, MarketDepth, PriceLevel};

impl exchange_interface::market_data::LastTrade {
    pub fn to_rest_api_last_trade(&self, instrument_id: i64) -> LastTrade {
        LastTrade {
            sequence_number: self.sequence_number,
            instrument_id,
            create_time: self.create_time,
            price: self.price,
            quantity: self.quantity,
        }
    }
}

impl exchange_interface::market_data::MarketDepth {
    pub fn to_rest_api_position(&self, instrument_id: i64) -> MarketDepth {
        let buys = self.buys.iter().map(|buy| { buy.to_rest_api_price_level() } ).collect();
        let sells = self.sells.iter().map(|sell| { sell.to_rest_api_price_level() } ).collect();

        MarketDepth {
            sequence_number: self.sequence_number,
            instrument_id,
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
