use serde::{Deserialize, Serialize};
use crate::rest_api::customer::Customer;
use crate::rest_api::trading::Instrument;

#[derive(Debug, Deserialize, Serialize)]
pub struct Position {
    pub instrument_id: u64,
    pub quantity: i32,
    pub cost: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Balance {
    pub cash: f64,
}
