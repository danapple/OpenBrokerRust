#[macro_use]
extern crate actix_web;

use crate::trade_handling::trade_handling::{handle_depth, handle_execution, handle_order_state, handle_last_trade};
use std::error::Error;
use actix_web::{
    dev::ServiceResponse,
    http::header,
    middleware,
    middleware::{ErrorHandlerResponse, ErrorHandlers}, web::ThinData, App, HttpServer,
    Result,
};
use std::io;
use std::sync::Arc;
use crate::config::BrokerConfig;

use confik::{Configuration as _, EnvSource};
use tokio_postgres::NoTls;

use env_logger::Env;
use log::{error, info};
use dotenv::dotenv;

mod constants;

mod rest_api;
mod exchange_interface;

use rest_api::account_api;
use rest_api::trading_api;
use crate::exchange_interface::exchange_client::ExchangeClient;
use instrument_manager::InstrumentManager;
use crate::access_control::AccessControl;
use crate::exchange_interface::trading::{Execution, ExecutionsTopicWrapper, OrderState};
use crate::exchange_interface::websocket_client::ExchangeWebsocketClient;
use crate::persistence::dao;
use crate::vetting::all_pass_vetter::{AllPassVetter};
use crate::websockets::client::StompMessage;
use crate::websockets::listener::{start_websocket_listener};

mod entities;
mod config;
mod persistence;
pub(crate) mod instrument_manager;
mod time;
mod access_control;
mod vetting;
mod websockets;
mod trade_handling;

fn add_error_header<B>(mut res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {

    res.response_mut().headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("Error"),
    );

    Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))

}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    let config = match BrokerConfig::builder()
        .override_with(EnvSource::new())
        .try_build() {
        Ok(x) => x,
        Err(_) => todo!(),
    };

    env_logger::init_from_env(Env::default().default_filter_or(config.log_level.clone()));

    let exchange_websocket_client = ExchangeWebsocketClient::new(config.clone(),
                                                                 handle_execution, handle_order_state,
                                                                 handle_depth,handle_last_trade);

    match start_websocket_listener(&config).await {
        Ok(_) => {},
        Err(_) => todo!(),
    };

    exchange_websocket_client.start_exchange_websockets().await;

    let pool = match config.pg.create_pool(None, NoTls) {
        Ok(x) => x,
        Err(_) => todo!(),
    };

    info!("About to add instruments");

    // TODO loop in another thread until instruments are retrieved
    let base_exchange_client = Arc::new(ExchangeClient::new(&config));
    let base_instruments = base_exchange_client.clone().get_instruments().await;
    let mut instrument_manager = InstrumentManager::new();

    for instrument in base_instruments.instruments.values() {
        info!("Adding instrument: {} for exchange {}", instrument.instrument_id, config.exchange_url);
        instrument_manager.add_instrument(instrument.instrument_id, base_exchange_client.clone());
    }
    info!("Done adding instruments");

    let dao = dao::Dao::new(pool);
    
    let access_control = AccessControl::new();

    let vetter = AllPassVetter::new();
    
    HttpServer::new(move || {
        App::new()
            .app_data(ThinData(instrument_manager.clone()))
            .app_data(ThinData(dao.clone()))
            .app_data(ThinData(access_control.clone()))
            .app_data(ThinData(vetter.clone()))
            .wrap(middleware::Logger::default())
            .wrap(ErrorHandlers::new().default_handler(add_error_header))
            .service(trading_api::get_order)
            .service(trading_api::get_orders)
            .service(trading_api::preview_order)
            .service(trading_api::submit_order)
            .service(trading_api::cancel_order)
            .service(account_api::get_positions)
            .service(account_api::get_balance)
    })
        .bind(config.server_addr)?
        .run()
        .await
}

