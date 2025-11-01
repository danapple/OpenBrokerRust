use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct LastTrade {
    pub version_number: i64,
    pub instrument_key: String,
    pub create_time: i64,
    pub price: f32,
    pub quantity: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MarketDepth {
    pub version_number: i64,
    pub instrument_key: String,
    pub create_time: i64,
    pub buys: Vec<PriceLevel>,
    pub sells: Vec<PriceLevel>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PriceLevel {
    pub price: f32,
    pub quantity: i32,
}
