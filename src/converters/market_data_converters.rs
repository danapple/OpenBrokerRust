use crate::dtos::market_data::{LastTrade, MarketDepth, PriceLevel};
use crate::exchange_interface;

impl exchange_interface::market_data::LastTrade {
    pub fn to_rest_api_last_trade(&self, 
                                  instrument_key: String) -> LastTrade {
                LastTrade {
            version_number: self.sequence_number,
            instrument_key,
            create_time: self.create_time,
            price: self.price,
            quantity: self.quantity,
        }
    }
}

impl exchange_interface::market_data::MarketDepth {
    pub fn to_rest_api_market_depth(&self, 
                                    instrument_key: String) -> MarketDepth {
        let buys = self.buys.iter().map(|buy| { buy.to_rest_api_price_level() } ).collect();
        let sells = self.sells.iter().map(|sell| { sell.to_rest_api_price_level() } ).collect();

        MarketDepth {
            version_number: self.sequence_number,
            instrument_key,
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
