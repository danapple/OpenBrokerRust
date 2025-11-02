use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[derive(Clone)]
pub enum AssetClass {
    #[serde(rename = "EQUITY")]
    Equity,
    #[serde(rename = "OPTION")]
    Option,
    #[serde(rename = "COMMODITY")]
    Commodity,
    #[serde(rename = "FUTURE")]
    Future,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[derive(Clone)]
pub enum InstrumentStatus {
    #[serde(rename = "ACTIVE")]
    Active,
    #[serde(rename = "INACTIVE")]
    Inactive,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Instrument {
    #[serde(rename = "instrumentId")]
    pub instrument_id: i64,
    pub status: InstrumentStatus,
    pub symbol: String,
    #[serde(rename = "assetClass")]
    pub asset_class: AssetClass,
    pub description: String,
    #[serde(rename = "expirationTime")]
    pub expiration_time: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Instruments {
    #[serde(rename = "instruments")]
    pub instruments: HashMap<i64, Instrument>,
}