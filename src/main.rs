#[macro_use]
extern crate actix_web;

use actix_files as fs;

use crate::config::BrokerConfig;

use actix_cors::Cors;
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{dev::ServiceResponse, http::header, middleware, middleware::{ErrorHandlerResponse, ErrorHandlers}, web::ThinData, App, HttpServer, Result};
use confik::{Configuration as _, EnvSource};
use std::io;
use tokio_postgres::NoTls;

use dotenv::dotenv;
use env_logger::Env;

mod constants;

mod rest_api;
mod admin_api;
mod auth;
mod exchange_interface;

use crate::access_control::AccessControl;
use crate::auth::{account_pages, auth_api, auth_ui};
use crate::persistence::dao::Dao;
use crate::rest_api::instrument_api;
use crate::validator::validator::Validator;
use crate::vetting::all_pass_vetter::AllPassVetter;
use crate::websockets::server::WebSocketServer;
use crate::websockets::ws_handler;
use instrument_manager::InstrumentManager;
use rest_api::account_api;
use rest_api::balance_position_api;
use rest_api::order_api;

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
mod converters;
mod dtos;
mod validator;

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
    let web_socket_server = WebSocketServer::new();

    let dao = Dao::new(pool);

    let mut instrument_manager = InstrumentManager::new(dao.clone(), web_socket_server.clone());
    match instrument_manager.initialize().await {
        Ok(_) => { },
        Err(init_error) => panic!("Could not initialize instrument manager: {}", init_error),
    };

    let oconfig = config.clone();

    let access_control = AccessControl::new();

    let vetter = AllPassVetter::new();

    let validator = Validator::new(instrument_manager.clone());

    let secret_key = Key::from(config.session_key.as_bytes());
    let redis_store = match RedisSessionStore::new(config.redis_addr)
        .await {
        Ok(redis_store) => redis_store,
        Err(redis_error) => panic!("Could not create redis store: {}", redis_error),
    };

    HttpServer::new(move || {
        App::new()
            .app_data(ThinData(instrument_manager.clone()))
            .app_data(ThinData(dao.clone()))
            .app_data(ThinData(access_control.clone()))
            .app_data(ThinData(vetter.clone()))
            .app_data(ThinData(validator.clone()))
            .app_data(ThinData(web_socket_server.clone()))
            .app_data(ThinData(oconfig.clone()))
            .wrap(middleware::Logger::default())
            .wrap(
                SessionMiddleware::new(
                    redis_store.clone(),
                    secret_key.clone(),
                )
            )
            .wrap(ErrorHandlers::new().default_handler(add_error_header))
            .wrap(
                Cors::permissive()
                    .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
                    .max_age(3600)
                    )
            .service(order_api::get_order)
            .service(order_api::get_orders)
            .service(order_api::preview_order)
            .service(order_api::submit_order)
            .service(order_api::cancel_order)
            .service(balance_position_api::get_positions)
            .service(balance_position_api::get_balance)
            .service(account_api::get_accounts)
            .service(account_pages::welcome)
            .service(account_pages::register)
            .service(account_pages::login)
            .service(auth_ui::register_ui)
            .service(auth_ui::login_ui)
            .service(auth_api::login_api)
            .service(account_pages::logout)
            .service(admin_api::offer_admin::create_offer)
            .service(admin_api::instrument_admin::create_exchange)
            .service(admin_api::instrument_admin::load_exchange_instruments)
            .service(instrument_api::get_instruments)
            .service(ws_handler::ws_setup)
            .service(fs::Files::new("/app", "./resources/static/app")
                         .show_files_listing()
                         .index_file("app.html")
                         .use_last_modified(true),)
         })
        .bind(config.server_addr)?
        .run()
        .await
}
