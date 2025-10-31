use crate::access_control::AccessControl;
use crate::constants::{ACCOUNT_UPDATE_QUEUE_NAME, APPLICATION_JSON};
use crate::entities::trading::OrderLeg;
use crate::instrument_manager::{Instrument, InstrumentManager};
use crate::persistence::dao::{Dao, DaoError};
use crate::rest_api::account::Privilege;
use crate::rest_api::actor::Power;
use crate::rest_api::base_api;
use crate::rest_api::base_api::{log_dao_error_and_return_500, log_text_error_and_return_500, send_order_state};
use crate::rest_api::offer::Offer;
use crate::rest_api::trading::{is_order_status_open, Order, OrderState, OrderStatus, VettingResult};
use crate::rest_api::trading_converters::order_status_to_rest_api_order_status;
use crate::time::current_time_millis;
use crate::{entities, exchange_interface};
use actix_session::Session;
use actix_web::web::{Json, Path, ThinData};
use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::Error;
use log::{error, info, warn};
use uuid::Uuid;

#[post("/admin/offer")]
pub async fn create_offer(dao: ThinData<Dao>,
                          access_control: ThinData<AccessControl>,
                          session: Session,
                          offer_code: Json<Offer>,
) -> HttpResponse {
    info!("create_offer_code called");

    let allowed: bool = match access_control.is_admin_allowed(&session, Power::All).await {
        Ok(allowed) => allowed,
        Err(error) => {
            error!("Failed while checking admin access: {}", error.to_string());
            return HttpResponse::InternalServerError().finish();
        }
    };
    if !allowed {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = match dao.get_connection().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),

    };
    match txn.save_offer(offer_code.to_entities_offer()).await {
        Ok(_) => {}
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    match txn.commit().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    HttpResponse::Ok().finish()
}

