use crate::constants::ACCOUNT_UPDATE_QUEUE_NAME;
use crate::entities::trading::OrderState;
use crate::persistence::dao::DaoError;
use crate::websockets::server::WebSocketServer;
use actix_web::web::ThinData;
use actix_web::{HttpRequest, HttpResponse};
use log::error;

pub fn get_api_key(req: HttpRequest,) -> Option<String> {
    match req.cookie("api_key") {
        Some(cookie) => {
            Some(cookie.value().to_string())
        }
        None => {
            None
        }
    }
}

pub fn send_order_state(web_socket_server: &mut ThinData<WebSocketServer>, account_key: &String, order_state: &OrderState) {
    let account_update = crate::trade_handling::updates::AccountUpdate {
        balance: None,
        position: None,
        trade: None,
        order_state: Some(order_state.to_rest_api_order_state(account_key.as_str())),
    };
    web_socket_server.send_account_message(account_key.as_str(), ACCOUNT_UPDATE_QUEUE_NAME, &account_update);
}


pub fn log_dao_error_and_return_500(dao_error: DaoError) -> HttpResponse {
    error!("DaoError: {}", dao_error);
    HttpResponse::InternalServerError().finish()
}

pub fn log_text_error_and_return_500(error: String) -> HttpResponse {
    error!("Error: {}", error);
    HttpResponse::InternalServerError().finish()
}