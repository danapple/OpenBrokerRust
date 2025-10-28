#[macro_use]
extern crate actix_web;

use actix_files as fs;

use crate::config::BrokerConfig;
use crate::trade_handling::execution_handling::handle_execution;
use crate::trade_handling::order_state_handling::handle_order_state;

use actix_cors::Cors;
use actix_web::{dev::ServiceResponse, http::header, middleware, middleware::{ErrorHandlerResponse, ErrorHandlers}, web, web::ThinData, App, HttpServer, Result};
use confik::{Configuration as _, EnvSource};
use std::io;
use std::sync::Arc;
use tokio_postgres::NoTls;

use dotenv::dotenv;
use env_logger::Env;
use log::{error, info};

mod constants;

mod rest_api;
mod exchange_interface;

use crate::access_control::AccessControl;
use crate::exchange_interface::exchange_client::ExchangeClient;
use crate::exchange_interface::trading::{Execution, ExecutionsTopicWrapper, OrderState};
use crate::exchange_interface::websocket_client::ExchangeWebsocketClient;
use crate::market_data::receiver::{handle_depth, handle_last_trade};
use crate::persistence::dao;
use crate::vetting::all_pass_vetter::AllPassVetter;
use crate::websockets::{server, ws_handler};
use instrument_manager::InstrumentManager;
use rest_api::account_api;
use rest_api::balance_position_api;
use rest_api::trading_api;

mod entities;
mod config;
mod persistence;
pub(crate) mod instrument_manager;
mod time;
mod access_control;
mod vetting;
mod websockets;
mod trade_handling;
mod market_data;

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
        Ok(config) => config,
        Err(build_error) => panic!("Could not create BrokerConfig: {}", build_error),
    };

    env_logger::init_from_env(Env::default().default_filter_or(config.log_level.clone()));

    let pool = match config.pg.create_pool(None, NoTls) {
        Ok(pool) => pool,
        Err(pool_error) => panic!("Could not create database connection pool: {}", pool_error),
    };
    let dao = dao::Dao::new(pool);

    info!("About to add instruments");
    // TODO loop in another thread until instruments are retrieved
    let base_exchange_client = Arc::new(ExchangeClient::new(&config));
    let base_instruments = match base_exchange_client.clone().get_instruments().await {
        Ok(base_instruments) => base_instruments,
        Err(instrument_error) => todo!("Should retry getting instruments from the exchange: {}", instrument_error),
    };

    let mut instrument_manager = InstrumentManager::new();

    for instrument in base_instruments.instruments.values() {
        info!("Adding instrument: {} for exchange {}", instrument.instrument_id, config.exchange_url);
        instrument_manager.add_instrument(instrument.instrument_id, base_exchange_client.clone());
    }
    info!("Done adding instruments");

    let web_socket_server = server::WebSocketServer::new();

    let exchange_websocket_client = ExchangeWebsocketClient::new(config.clone(),
                                                                 dao.clone(),
                                                                 web_socket_server.clone(),
                                                                 instrument_manager.clone(),
                                                                 handle_execution, handle_order_state,
                                                                 handle_depth, handle_last_trade);
    exchange_websocket_client.start_exchange_websockets().await;
    
    let access_control = AccessControl::new(dao.clone());

    let vetter = AllPassVetter::new();
    
    HttpServer::new(move || {
        App::new()
            .app_data(ThinData(instrument_manager.clone()))
            .app_data(ThinData(dao.clone()))
            .app_data(ThinData(access_control.clone()))
            .app_data(ThinData(vetter.clone()))
            .app_data(ThinData(web_socket_server.clone()))
            .wrap(middleware::Logger::default())
            .wrap(ErrorHandlers::new().default_handler(add_error_header))

            .wrap(
                Cors::permissive()
                    .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
                    .max_age(3600)
                    )
            .service(trading_api::get_order)
            .service(trading_api::get_orders)
            .service(trading_api::preview_order)
            .service(trading_api::submit_order)
            .service(trading_api::cancel_order)
            .service(balance_position_api::get_positions)
            .service(balance_position_api::get_balance)
            .service(account_api::get_accounts)
            .service(web::resource("/ws").route(web::get().to(ws_handler::ws_setup)))
            .service(fs::Files::new("/", "./resources/static")
                         .show_files_listing()
                         .index_file("index.html")
                         .use_last_modified(true),)
         })
        .bind(config.server_addr)?
        .run()
        .await
}

