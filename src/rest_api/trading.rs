use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::SystemTime;
use tokio_postgres::types::{FromSql, ToSql};

#[derive(Debug, Deserialize, Serialize, Clone, ToSql, FromSql, PartialEq)]
pub enum OrderStatus {
    Rejected,
    Pending,
    Open,
    Filled,
    PendingCancel,
    Canceled,
    Expired,
}

impl Display for OrderStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for OrderStatus {
    type Err = ();
    fn from_str(input: &str) -> Result<OrderStatus, Self::Err> {
        match input {
            "Rejected"  => Ok(OrderStatus::Rejected),
            "Pending"  => Ok(OrderStatus::Pending),
            "Open"  => Ok(OrderStatus::Open),
            "Filled" => Ok(OrderStatus::Filled),
            "PendingCancel" => Ok(OrderStatus::PendingCancel),
            "Canceled" => Ok(OrderStatus::Canceled),
            "Expired" => Ok(OrderStatus::Expired),
            _  => Err(()),
        }
    }
}

pub fn is_order_status_open(order_status: &OrderStatus) -> bool {
    match order_status {
        OrderStatus::Rejected => false,
        OrderStatus::Pending => true,
        OrderStatus::Open => true,
        OrderStatus::Filled => false,
        OrderStatus::PendingCancel => true,
        OrderStatus::Canceled => false,
        OrderStatus::Expired => false,
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[derive(Clone)]
pub struct Instrument {
    pub instrument_id: i64,
    pub value_factor: f32,
    pub underlying_instrument_id: i64,
    pub underlying_quantity: f32,
}

#[derive(Debug, Deserialize, Serialize)]
#[derive(Clone)]
pub struct OrderState {
    pub update_time: i64,
    pub order_status: OrderStatus,
    pub remaining_quantity: i32,
    pub order: Order,
    pub version_number: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[derive(Clone)]
pub struct Order {
    #[serde(default)]
    pub create_time: i64,
    pub ext_order_id: Option<String>,
    pub account_key: Option<String>,
    pub price: f32,
    pub quantity: i32,
    pub legs: Vec<OrderLeg>,
}

#[derive(Debug, Deserialize, Serialize)]
#[derive(Clone)]
pub struct OrderLeg {
    //#[serde(deserialize_with = "as_i64")]
    pub instrument_id: i64,
    pub ratio: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[derive(Clone)]
pub struct Trade {
    pub create_time: SystemTime,
    pub price: f32,
    pub quantity: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VettingResult {
    pub pass: bool
}