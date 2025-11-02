use crate::access_control::AccessControl;
use crate::constants::APPLICATION_JSON;
use actix_session::Session;
use actix_web::web::{Json, Path, ThinData};
use actix_web::HttpResponse;
use log::{error, info};
use std::collections::HashMap;
use std::string::ToString;
use uuid::Uuid;

use crate::instrument_manager::InstrumentManager;
use crate::persistence::dao::Dao;
use crate::rest_api::account::Privilege;
use crate::rest_api::base_api::{log_dao_error_and_return_500, log_text_error_and_return_500, send_order_state};
use crate::rest_api::exchange::InstrumentStatus;
use crate::rest_api::trading::{is_order_status_open, Order, OrderState, OrderStatus, VettingResult};
use crate::rest_api::trading_converters::order_status_to_rest_api_order_status;
use crate::time::current_time_millis;
use crate::vetting::all_pass_vetter::AllPassVetter;
use crate::websockets::server::WebSocketServer;
use crate::{entities, exchange_interface};

#[get("/accounts/{account_key}/orders")]
pub async fn get_orders(dao: ThinData<Dao>,
                        instrument_manager: ThinData<InstrumentManager>,
                        access_control: ThinData<AccessControl>,
                        session: Session,
                        path: Path<(String)>,
) -> HttpResponse {
    info!("get_orders called");
    let account_key = path.into_inner();

    let allowed: bool = match access_control.is_allowed_account_privilege(&session, &account_key, Privilege::Read) {
        Ok(allowed) => allowed,
        Err(error) => {
            error!("Failed while checking access: {}", error.to_string());
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
    let order_states = match txn.get_orders(&account_key).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    match txn.rollback().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };

    let mut api_order_states: HashMap<String, OrderState> = HashMap::new();
    for order_state in order_states {
        api_order_states.insert(order_state.0, order_state.1.to_rest_api_order_state(account_key.as_str(),
                                                                                     &instrument_manager));
    }
    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(api_order_states)
}

#[get("/accounts/{account_key}/orders/{ext_order_id}")]
pub async fn get_order(dao: ThinData<Dao>,
                       instrument_manager: ThinData<InstrumentManager>,
                       access_control: ThinData<AccessControl>,
                       session: Session,
                       path: Path<(String, String)>,) -> HttpResponse {
    let (account_key, ext_order_id) = path.into_inner();
    info!("get_order called for ext_order_id {ext_order_id}");
    let allowed: bool = match access_control.is_allowed_account_privilege(&session, &account_key, Privilege::Read) {
        Ok(allowed) => allowed,
        Err(error) => {
            error!("Failed while checking access: {}", error.to_string());
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

    let order_state_option = match txn.get_order_by_ext_order_id(&account_key, &ext_order_id).await {
        Ok(x) => x,
        Err(_) => return HttpResponse::NotFound().finish(),
    };
    match txn.rollback().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    match order_state_option {
        Some(x) => HttpResponse::Ok()
            .content_type(APPLICATION_JSON)
            .json(x.to_rest_api_order_state(account_key.as_str(), &instrument_manager)),
        None => HttpResponse::NotFound().finish()
    }
}


#[post("/accounts/{account_key}/previewOrder")]
pub async fn preview_order(dao: ThinData<Dao>,
                           access_control: ThinData<AccessControl>,
                           session: Session,
                           instrument_manager: ThinData<InstrumentManager>,
                           vetter: ThinData<AllPassVetter>,
                           path: Path<(String)>,
                           rest_api_order: Json<Order>) -> HttpResponse {
    let account_key = path.into_inner();
    let allowed: bool = match access_control.is_allowed_account_privilege(&session, &account_key, Privilege::Read) {
        Ok(allowed) => allowed,
        Err(error) => {
            error!("Failed while checking access: {}", error.to_string());
            return HttpResponse::InternalServerError().finish();
        }
    };
    if !allowed {
        return HttpResponse::Forbidden().finish();
    }

    let vetting_result = match vetter.vet_order(&rest_api_order).await {
        Ok(x) => x,
        Err(vetting_error) => {
            error!("{}", vetting_error);
            return HttpResponse::InternalServerError().finish()
        },
    };
    let rest_api_vetting_result = VettingResult {
        pass: vetting_result.pass,
    };
    HttpResponse::Ok().json(rest_api_vetting_result)
}

#[post("/accounts/{account_key}/orders")]
pub async fn submit_order(dao: ThinData<Dao>,
                          instrument_manager: ThinData<InstrumentManager>,
                          access_control: ThinData<AccessControl>,
                          session: Session,
                          vetter: ThinData<AllPassVetter>,
                          mut web_socket_server: ThinData<WebSocketServer>,
                          path: Path<(String)>,
                          mut rest_api_order: Json<Order>) -> HttpResponse {
    info!("submit_order called");

    let account_key = path.into_inner();

    let allowed: bool = match access_control.is_allowed_account_privilege(&session, &account_key, Privilege::Read) {
        Ok(allowed) => allowed,
        Err(error) => {
            error!("Failed while checking access: {}", error.to_string());
            return HttpResponse::InternalServerError().finish();
        }
    };
    if !allowed {
        return HttpResponse::Forbidden().finish();
    }

    let vetting_result = match vetter.vet_order(&rest_api_order).await {
        Ok(x) => x,
        Err(vetting_error) => {
            error!("Vetting error: {}", vetting_error);
            return HttpResponse::InternalServerError().finish()
        },
    };
    if !vetting_result.pass {
        let rest_api_vetting_result = VettingResult {
            pass: vetting_result.pass,
        };
        return HttpResponse::PreconditionFailed().json(rest_api_vetting_result);
    }

    let first_leg_instrument_key = match rest_api_order.legs.first() {
        Some(leg0) => leg0.instrument_key.clone(),
        None => return HttpResponse::PreconditionFailed().json("no order legs")
    };

    let instrument_result = instrument_manager.get_instrument_by_key(&first_leg_instrument_key);
    let instrument_option = match instrument_result {
        Ok(instrument_option) => instrument_option,
        Err(instrument_error) => {
            error!("Could not get instrument: {}", instrument_error);
            return HttpResponse::PreconditionFailed().finish()
        }

    };
    let instrument = match instrument_option {
        Some(instrument) => instrument,
        None => {
            return HttpResponse::PreconditionFailed().json(format!("instrument {} is unknown", first_leg_instrument_key))
        }
    };

    let ext_order_id = rest_api_order.ext_order_id.clone().unwrap_or_else(|| Uuid::new_v4().simple().to_string());
    rest_api_order.ext_order_id = Some(ext_order_id);
    rest_api_order.account_key = Some(account_key.clone());

    let mut db_connection = match dao.get_connection().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),

    };
    let account = match txn.get_account_by_account_key(&account_key.clone()).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let exchange_order_result = rest_api_order.to_exchange_order(&instrument_manager);
    let exchange_order = match exchange_order_result {
        Ok(exchange_order) => exchange_order,
        Err(err) => return log_text_error_and_return_500(err.to_string())
    };
    let entities_order_result = rest_api_order.to_entities_order(&account, exchange_order.client_order_id.clone(), &instrument_manager);
    let entities_order = match entities_order_result {
        Ok(entities_order) => entities_order,
        Err(err) => return log_text_error_and_return_500(err.to_string())
    };
    let mut order_state = entities::trading::OrderState {
        update_time: current_time_millis(),
        order_status: OrderStatus::Pending,
        order: entities_order,
        version_number: 0,
    };

    match instrument.status {
        InstrumentStatus::Active => {}
        InstrumentStatus::Inactive => {
            order_state.order_status = OrderStatus::Rejected
        }
    }

    if instrument.expiration_time < current_time_millis() {
        order_state.order_status = OrderStatus::Rejected
    }

    let mut db_connection = match dao.get_connection().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    order_state = match txn.save_order(order_state).await {
        Ok(x) => x,
        Err(dao_error) => {
            return log_dao_error_and_return_500(dao_error);
        },
    };
    match txn.commit().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    send_order_state(&mut web_socket_server, &instrument_manager, &account_key, &order_state);

    if order_state.order_status != OrderStatus::Pending {
        return HttpResponse::PreconditionFailed().json(format!("instrument {} is not available for trading", first_leg_instrument_key))
    }

    let exchange_client = match instrument_manager.get_exchange_client_for_instrument(&instrument) {
        Ok(exchange_client) => exchange_client,
        Err(instrument_error) => {
            error!("Could not get exchange for instrument {}: {}", instrument.instrument_id, instrument_error);
            return HttpResponse::PreconditionFailed().finish()
        }
    };

    let exchange_order_state = match exchange_client.submit_order(exchange_order).await {
        Ok(exchange_order_state) => exchange_order_state,
        Err(submit_order_error) => {
            error!("submit_order_error: {}", submit_order_error);
            return HttpResponse::InternalServerError().finish()
        },
    };

    // We'll get async notifications for all status updates other than Rejected
    if exchange_order_state.order_status == exchange_interface::trading::OrderStatus::Rejected {
        order_state.order_status = OrderStatus::Rejected;
        order_state.update_time = current_time_millis();

        let txn = match dao.begin(&mut db_connection).await {
            Ok(x) => x,
            Err(dao_error) => return log_dao_error_and_return_500(dao_error),
        };
        match txn.update_order(&mut order_state).await {
            Ok(x) => x,
            Err(dao_error) => return log_dao_error_and_return_500(dao_error),
        };
        match txn.commit().await {
            Ok(x) => x,
            Err(dao_error) => return log_dao_error_and_return_500(dao_error),
        };
        send_order_state(&mut web_socket_server, &instrument_manager, &account_key, &order_state);
    }

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(order_state.to_rest_api_order_state(account_key.as_str(), &instrument_manager))
}

#[delete("/accounts/{account_key}/orders/{ext_order_id}")]
pub async fn cancel_order(dao: ThinData<Dao>,
                          mut web_socket_server: ThinData<WebSocketServer>,
                          access_control: ThinData<AccessControl>,
                          session: Session,
                          instrument_manager: ThinData<InstrumentManager>,
                          path: Path<(String, String)>,) -> HttpResponse {
    let (account_key, ext_order_id) = path.into_inner();

    info!("cancel_order called for ext_order_id {ext_order_id}");
    let allowed: bool = match access_control.is_allowed_account_privilege(&session, &account_key, Privilege::Read) {
        Ok(allowed) => allowed,
        Err(error) => {
            error!("Failed while checking access: {}", error.to_string());
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

    let order_state_option = match txn.get_order_by_ext_order_id(&account_key, &ext_order_id).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };

    let mut order_state = match order_state_option {
        Some(x) => x,
        None => return HttpResponse::NotFound().finish(),
    };

    if !is_order_status_open(&order_state.order_status) {
        return HttpResponse::PreconditionFailed().finish();
    }

    order_state.order_status = OrderStatus::PendingCancel;
    match txn.update_order(&mut order_state).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    match txn.commit().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    send_order_state(&mut web_socket_server, &instrument_manager, &account_key, &order_state);

    let first_leg_instrument_id = match order_state.order.legs.first() {
        Some(leg0) => leg0.instrument_id,
        None => return HttpResponse::PreconditionFailed().json("no order legs")
    };

    let instrument_result = instrument_manager.get_instrument(first_leg_instrument_id);
    let instrument_option = match instrument_result {
        Ok(instrument_option) => instrument_option,
        Err(instrument_error) => {
            error!("Could not get instrument: {}", instrument_error);
            return HttpResponse::PreconditionFailed().finish()
        }
    };
    let instrument = match instrument_option {
        Some(instrument) => instrument,
        None => {
            return HttpResponse::PreconditionFailed().json("instrument 0 is unknown")
        }
    };

    let exchange_client = match instrument_manager.get_exchange_client_for_instrument(&instrument) {
        Ok(exchange_client) => exchange_client,
        Err(instrument_error) => {
            error!("Could not get exchange: {}", instrument_error);
            return HttpResponse::PreconditionFailed().finish()
        }
    };

    let exchange_order_state = match exchange_client.cancel_order(order_state.clone().order.client_order_id).await {
        Ok(exchange_order_state) => exchange_order_state,
        Err(cancel_order_error) => {
            error!("cancel_order_error: {}", cancel_order_error);
            return HttpResponse::InternalServerError().finish()
        },
    };

    order_state.order_status = order_status_to_rest_api_order_status(exchange_order_state.order_status);
    order_state.update_time = current_time_millis();

    let txn = match dao.begin(&mut db_connection).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    match txn.update_order(&mut order_state).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    match txn.commit().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    send_order_state(&mut web_socket_server, &instrument_manager, &account_key, &order_state);

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(order_state.to_rest_api_order_state(account_key.as_str(), &instrument_manager))
}
