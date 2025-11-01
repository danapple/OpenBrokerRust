use crate::access_control::AccessControl;
use crate::constants::APPLICATION_JSON;
use crate::instrument_manager::InstrumentManager;
use crate::rest_api::base_api::log_text_error_and_return_500;
use actix_session::Session;
use actix_web::web::ThinData;
use actix_web::HttpResponse;
use log::{error, info};
use std::collections::HashMap;

#[get("/instruments")]
pub async fn get_instruments(access_control: ThinData<AccessControl>,
                             instrument_manager: ThinData<InstrumentManager>,

                             session: Session,) -> HttpResponse {
    info!("get_instruments called");
    let allowed = match access_control.is_allowed(&session) {
        Ok(allowed) => allowed,
        Err(error) => {
            error!("Failed while checking access: {}", error.to_string());
            return HttpResponse::InternalServerError().finish();
        }
    };
    if !allowed {
        return HttpResponse::Forbidden().finish();
    }
    let instruments = match instrument_manager.get_instruments() {
        Ok(x) => x,
        Err(get_error) => return log_text_error_and_return_500(get_error.to_string()),
    };
    let mut rest_api_instruments = HashMap::new();
    for instrument in instruments.values() {
        let exchange = match instrument_manager.get_exchange_for_instrument(instrument) {
            Ok(exchange) => exchange,
            Err(get_error) => return log_text_error_and_return_500(get_error.to_string()),
        };
        rest_api_instruments.insert(instrument.instrument_key.clone(), instrument.to_rest_api_instrument(&exchange));
    }

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(rest_api_instruments)
}
