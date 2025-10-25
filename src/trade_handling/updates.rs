use crate::rest_api::account::{Balance, Position};
use crate::rest_api::trading::OrderState;
use crate::rest_api::trading::Trade;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountUpdate {
    pub position: Option<Position>,
    pub balance: Option<Balance>,
    pub trade: Option<Trade>,
    pub order_state: Option<OrderState>
}