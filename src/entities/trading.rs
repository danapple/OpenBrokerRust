pub(crate) use crate::rest_api::trading::OrderStatus;

// #[derive(Clone)]
// pub struct Instrument {
//     pub instrument_id: u64,
//     pub exchange_instrument_id: u64,
//     pub value_factor: f64,
//     pub underlying_instrument_id: u64,
//     pub underlying_quantity: f64,
// }

#[derive(Clone)]
pub struct OrderState {
    pub order: Order,
    pub update_time: i64,
    pub order_status: OrderStatus,
    pub version_number: i64,
}

impl OrderState {
    pub fn get_order_mut(&mut self) -> &mut Order {
        &mut self.order
    }
}

#[derive(Clone)]
pub struct Order {
    pub order_id: i64,
    pub account_id: i32,
    pub order_number: i32,
    pub ext_order_id: String,
    pub client_order_id: String,
    pub create_time: i64,
    pub price: f32,
    pub quantity: i32,
    pub legs: Vec<OrderLeg>,
}

impl Order {
    pub fn add_leg(&mut self, leg: OrderLeg) {
        self.legs.push(leg);
    }
}

#[derive(Clone)]
pub struct OrderLeg {
    pub order_leg_id: i64,
    pub instrument_id: i64,
    pub ratio: i32,
}

#[derive(Clone)]
pub struct Trade {
    pub trade_id: i64,
    pub create_time: i64,
    pub order_leg: OrderLeg,
    pub price: f32,
    pub quantity: i32,
}
