use crate::dtos::account::Privilege;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Position {
    pub position_id: i64,
    pub account_id: i32,
    pub instrument_id: i64,
    pub quantity: i32,
    pub cost: f32,
    pub closed_gain: f32,
    pub update_time: i64,
    pub version_number: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Account {
    pub account_id: i32,
    pub account_key: String,
    pub account_number: String,
    pub account_name: String,
}

#[derive(Clone)]
pub struct Balance {
    pub balance_id: i32,
    pub account_id: i32,
    pub cash: f32,
    pub update_time: i64,
    pub version_number: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Access {
    pub actor_id: i32,
    pub account_id: i32,
    pub nickname: String,
    pub privilege: Privilege,
}

