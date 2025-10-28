use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct LastTrade {
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "sequenceNumber")]
    pub sequence_number: i64,
    #[serde(rename = "instrumentId")]
    pub instrument_id: i64,
    #[serde(rename = "createTime")]
    pub create_time: i64,
    pub price: f32,
    pub quantity: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MarketDepth {
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "sequenceNumber")]
    pub sequence_number: i64,
    #[serde(rename = "instrumentId")]
    pub instrument_id: i64,
    #[serde(rename = "createTime")]
    pub create_time: i64,
    pub buys: Vec<PriceLevel>,
    pub sells: Vec<PriceLevel>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PriceLevel {
    pub price: f32,
    pub quantity: i32,
}