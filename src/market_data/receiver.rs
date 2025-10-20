use log::info;
use crate::exchange_interface::trading::{LastTrade, MarketDepth};

pub fn handle_depth(depth: MarketDepth) {
    info!("Depth: {:?}", depth);

}

pub fn handle_last_trade(last_trade: LastTrade) {
    info!("Last Trade: {:?}", last_trade);
}