use log::info;
use crate::exchange_interface::trading::{LastTrade, MarketDepth};
use crate::persistence::dao::Dao;

pub fn handle_depth(dao: &Dao, depth: MarketDepth) {
    info!("Depth: {:?}", depth);
}

pub fn handle_last_trade(dao: &Dao, last_trade: LastTrade) {
    info!("Last Trade: {:?}", last_trade);
}