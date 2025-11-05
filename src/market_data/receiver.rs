use crate::exchange_interface::market_data::{LastTrade, MarketDepth};
use crate::instrument_manager::InstrumentManager;
use crate::persistence::dao::Dao;
use crate::websockets::server::WebSocketServer;
use log::{debug, info, warn};

pub fn handle_depth(web_socket_server: &WebSocketServer, 
                    instrument_manager: &InstrumentManager, 
                    depth: MarketDepth) {
    debug!("Depth: {:?}", depth);
    let (destination, instrument_key) = match compute_destination(instrument_manager, "depth", depth.instrument_id)
    {
        Ok(destination) => destination,
        Err(compute_err) => {
            warn!("Error computing destination: {:?}", compute_err);
            return;
        }
    };
    web_socket_server.clone().send_retained_message(destination, &depth.to_rest_api_market_depth(instrument_key));
}

pub fn handle_last_trade(web_socket_server: &WebSocketServer, 
                         instrument_manager: &InstrumentManager, 
                         last_trade: LastTrade) {
    debug!("Last Trade: {:?}", last_trade);
    let (destination, instrument_key) = match compute_destination(instrument_manager, "last_trade", last_trade.instrument_id)
    {
        Ok(destination) => destination,
        Err(compute_err) => {
            warn!("Error computing destination: {:?}", compute_err);
            return;
        }
    };
    web_socket_server.clone().send_retained_message(destination, &last_trade.to_rest_api_last_trade(instrument_key));
}

fn compute_destination(instrument_manager: &InstrumentManager, 
                       scope: &str, 
                       exchange_instrument_id: i64) -> Result<(String, String), anyhow::Error> {
    let instrument_option = instrument_manager.get_instrument_by_exchange_instrument_id(exchange_instrument_id)?;
    let instrument = match instrument_option {
        Some(instrument) => instrument,
        None => return Err(anyhow::anyhow!("Could not find instrument for exchange instrument_id {}", exchange_instrument_id))
    };
    let destination = format!("/markets/{}/{}", instrument.instrument_key, scope);
    Ok((destination, instrument.instrument_key))
}