use crate::dtos::account::{Balance, Position};
use crate::dtos::order::OrderState;
use crate::dtos::order::Trade;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountUpdate {
    pub position: Option<Position>,
    pub balance: Option<Balance>,
    pub trade: Option<Trade>,
    pub order_state: Option<OrderState>
}