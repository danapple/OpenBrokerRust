use crate::constants::ACCOUNT_UPDATE_QUEUE_NAME;
use crate::entities::order::OrderState;
use crate::instrument_manager::InstrumentManager;
use crate::persistence::dao::DaoError;
use crate::websockets::server::WebSocketServer;
use actix_web::web::ThinData;
use actix_web::HttpResponse;
use log::error;


pub fn log_dao_error_and_return_500(dao_error: DaoError) -> HttpResponse {
    error!("DaoError: {}", dao_error);
    HttpResponse::InternalServerError().finish()
}

pub fn log_text_error_and_return_500(error: String) -> HttpResponse {
    error!("Error: {}", error);
    HttpResponse::InternalServerError().finish()
}