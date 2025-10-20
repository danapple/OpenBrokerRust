use log::info;
use crate::exchange_interface::trading::{Execution, OrderState};

pub fn handle_order_state(order_state: OrderState) {
    info!("Order state: {:?}", order_state);
}


pub fn handle_execution(execution: Execution) {
    info!("Execution: {:?}", execution);
}
