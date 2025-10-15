use std::collections::HashMap;
use std::iter::Map;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use crate::rest_api;

#[derive(Debug, Deserialize, Serialize)]
#[derive(Clone)]
pub enum OrderStatus {
    #[serde(rename = "OPEN")]
    Open,
    #[serde(rename = "CANCELED")]
    Canceled,
    #[serde(rename = "FILLED")]
    Filled,
    #[serde(rename = "EXPIRED")]
    Expired,
    #[serde(rename = "REJECTED")]
    Rejected,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderStates {
    #[serde(rename = "orderStates")]
    pub order_states: Vec<OrderState>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitOrders {
    pub orders: Vec<Order>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderState {
    #[serde(rename = "updateTime")]
    pub update_time: i64,
    #[serde(rename = "orderStatus")]
    pub order_status: OrderStatus,
    #[serde(rename = "remainingQuantity")]
    pub remaining_quantity: u32,
    pub order: Order
}

#[derive(Debug, Deserialize, Serialize)]
#[derive(Clone)]
pub struct Order {
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    #[serde(rename = "price")]
    pub price: f32,
    pub quantity: i32,
    pub legs: Vec<OrderLeg>,
}

#[derive(Debug, Deserialize, Serialize)]
#[derive(Clone)]
pub struct OrderLeg {
    #[serde(rename = "instrumentId")]
    pub instrument_id: i64,
    pub ratio: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Trade {
    #[serde(rename = "createTime")]
    pub create_time: SystemTime,
    pub price: f32,
    pub quantity: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Instrument {
    #[serde(rename = "instrumentId")]
    pub instrument_id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Instruments {
    #[serde(rename = "instruments")]
    pub instruments: HashMap<i64, Instrument>,
}
