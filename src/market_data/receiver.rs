use crate::exchange_interface::trading::{LastTrade, MarketDepth};
use crate::persistence::dao::Dao;
use crate::websockets::server::WebSocketServer;
use log::info;

pub fn handle_depth(dao: &Dao, web_socket_server: &WebSocketServer, depth: MarketDepth) {
    info!("Depth: {:?}", depth);
}

pub fn handle_last_trade(dao: &Dao, web_socket_server: &WebSocketServer, last_trade: LastTrade) {
    info!("Last Trade: {:?}", last_trade);
}