use crate::exchange_interface::market_data::{LastTrade, MarketDepth};
use crate::instrument_manager::InstrumentManager;
use crate::persistence::dao::Dao;
use crate::websockets::server::WebSocketServer;
use log::{debug, info, warn};

pub fn handle_depth(dao: &Dao, web_socket_server: &WebSocketServer, instrument_manager: &InstrumentManager, depth: MarketDepth) {
    debug!("Depth: {:?}", depth);
    let (destination, instrument_id) = match compute_destination(instrument_manager, "depth", depth.instrument_id)
    {
        Ok(destination) => destination,
        Err(compute_err) => {
            warn!("Error computing destination: {:?}", compute_err);
            return;
        }
    };
    web_socket_server.clone().send_retained_message(destination, &depth.to_rest_api_position(instrument_id));
}

pub fn handle_last_trade(dao: &Dao, web_socket_server: &WebSocketServer, instrument_manager: &InstrumentManager, last_trade: LastTrade) {
    debug!("Last Trade: {:?}", last_trade);
    let (destination, instrument_id) = match compute_destination(instrument_manager, "last_trade", last_trade.instrument_id)
    {
        Ok(destination) => destination,
        Err(compute_err) => {
            warn!("Error computing destination: {:?}", compute_err);
            return;
        }
    };
    web_socket_server.clone().send_retained_message(destination, &last_trade.to_rest_api_last_trade(instrument_id));
}

fn compute_destination(instrument_manager: &InstrumentManager, scope: &str, exchange_instrument_id: i64) -> Result<(String, i64), anyhow::Error> {
    let instrument_option = match instrument_manager.get_instrument_by_exchange_instrument_id(exchange_instrument_id) {
        Ok(instrument_option) => instrument_option,
        Err(lookup_error) => return Err(anyhow::anyhow!("Error when looking up exchange instrument_id {}: {}", exchange_instrument_id, lookup_error))
    };
    let instrument = match instrument_option {
        Some(instrument) => instrument,
        None => return Err(anyhow::anyhow!("Could not find instrument exchange instrument_id {}", exchange_instrument_id))
    };
    let instrument_id = instrument.instrument_id;
    let destination = format!("/markets/{}/{}", instrument_id, scope);
    Ok((destination, instrument_id))
}