use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
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
    pub ratio: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Execution {
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    #[serde(rename = "instrumentId")]
    pub instrument_id: i64,
    #[serde(rename = "createTime")]
    pub create_time: i64,
    pub price: f32,
    pub quantity: i32,
}

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
    pub price: f64,
    pub quantity: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExecutionsTopicWrapper {
    #[serde(rename = "orderState")]
    pub order_state: Option<OrderState>,
    pub execution: Option<Execution>
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
