use log::info;
use crate::exchange_interface::trading::{Execution, LastTrade, MarketDepth, OrderState};

pub fn handle_order_state(order_state: OrderState) {
    info!("Order state: {:?}", order_state);
}


pub fn handle_execution(execution: Execution) {
    info!("Execution: {:?}", execution);
}

pub fn handle_depth(depth: MarketDepth) {
    info!("Depth: {:?}", depth);

}

pub fn handle_last_trade(last_trade: LastTrade) {
    info!("Last Trade: {:?}", last_trade);
}