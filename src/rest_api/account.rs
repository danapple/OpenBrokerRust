use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub account_key: String,
    pub account_number: String,
    pub account_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Position {
    pub account_key: String,
    pub instrument_id: i64,
    pub quantity: i32,
    pub cost: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Balance {
    pub account_key: String,
    pub cash: f64,
}
