use std::collections::HashMap;
use std::string::ToString;
use crate::access_control::{AccessControl, Privilege};

use actix_web::HttpResponse;
use actix_web::web::{Json, Path, ThinData};
use log::info;
use crate::constants::APPLICATION_JSON;
use uuid::Uuid;

use crate::entities;
use crate::instrument_manager::InstrumentManager;
use crate::persistence::dao::Dao;
use crate::rest_api::base_api;
use crate::rest_api::converters::order_status_to_rest_api_order_status;
use crate::rest_api::trading::{is_order_status_open, Order, OrderState, OrderStatus, VettingResult};
use crate::time::current_time_millis;
use crate::vetting::all_pass_vetter::AllPassVetter;

#[get("/accounts/{account_key}/orders")]
pub async fn get_orders(dao: ThinData<Dao>,
                        access_control: ThinData<AccessControl>,
                        account_key: Path<(String,)>
) -> HttpResponse {
    info!("get_orders called");
    let account_key = &account_key.0.as_str().to_string();
    let customer_key = base_api::get_customer_key();
    if !access_control.is_allowed(account_key, &customer_key, Privilege::Read) {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;

    let order_states = match txn.get_orders(account_key).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    match txn.rollback().await {
        Ok(x) => x,
        Err(_) => todo!(),
    };

    let mut api_order_states: HashMap<String, OrderState> = HashMap::new();
    for order_state in order_states {
        api_order_states.insert(order_state.0, order_state.1.to_rest_api_order_state());
    }
    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(api_order_states)
}

#[get("/orders/{account_key}/orders/{ext_order_id}")]
pub async fn get_order(dao: ThinData<Dao>,
                       access_control: ThinData<AccessControl>,
                       account_key: Path<(String,)>,
                       ext_order_id: Path<(String,)>) -> HttpResponse {
    let account_key = &account_key.0.as_str().to_string();
    let ext_order_id = &ext_order_id.0.as_str().to_string();
    let customer_key = base_api::get_customer_key();
    info!("get_order called for ext_order_id {ext_order_id}");
    if !access_control.is_allowed(account_key, &customer_key, Privilege::Read) {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;

    let order_state_option = match txn.get_order(account_key, ext_order_id).await {
        Ok(x) => x,
        Err(_) => return HttpResponse::NotFound().finish(),
    };
    match txn.rollback().await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    match order_state_option {
        Some(x) => HttpResponse::Ok()
            .content_type(APPLICATION_JSON)
            .json(x.to_rest_api_order_state()),
        None => HttpResponse::NotFound().finish()
    }
}


#[post("/accounts/{account_key}/previewOrder")]
pub async fn preview_order(dao: ThinData<Dao>,
                           access_control: ThinData<AccessControl>,
                           instrument_manager: ThinData<InstrumentManager>,
                           vetter: ThinData<AllPassVetter>,
                           account_key: Path<(String,)>,
                           rest_api_order: Json<Order>) -> HttpResponse {
    let account_key = &account_key.0.as_str().to_string();
    let customer_key = base_api::get_customer_key();
    if !access_control.is_allowed(account_key, &customer_key, Privilege::Read) {
        return HttpResponse::Forbidden().finish();
    }

    let vetting_result = match vetter.vet_order(rest_api_order.clone()).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    let rest_api_vetting_result = VettingResult {
        pass: vetting_result.pass,
    };
    HttpResponse::Ok().json(rest_api_vetting_result)
}

#[post("/accounts/{account_key}/orders")]
pub async fn submit_order(dao: ThinData<Dao>,
                          access_control: ThinData<AccessControl>,
                          instrument_manager: ThinData<InstrumentManager>,
                          vetter: ThinData<AllPassVetter>,
                          account_key: Path<(String,)>,
                          rest_api_order: Json<Order>) -> HttpResponse {
    info!("submit_order called");
    let account_key = &account_key.0.as_str().to_string();
    let customer_key = base_api::get_customer_key();
    if !access_control.is_allowed(account_key, &customer_key, Privilege::Read) {
        return HttpResponse::Forbidden().finish();
    }

    let vetting_result = match vetter.vet_order(rest_api_order.clone()).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    if !vetting_result.pass {
        let rest_api_vetting_result = VettingResult {
            pass: vetting_result.pass,
        };
        return HttpResponse::PreconditionFailed().json(rest_api_vetting_result);
    }

    let instrument = instrument_manager.get_instrument(0);
    let ext_order_id = rest_api_order.ext_order_id.clone().unwrap_or_else(|| Uuid::new_v4().simple().to_string());
    let exchange_order = rest_api_order.to_exchange_order(instrument_manager);

    let entities_order = rest_api_order.to_entities_order(exchange_order.client_order_id.clone(), ext_order_id);
    let mut order_state = entities::trading::OrderState {
        update_time: current_time_millis(),
        order_status: OrderStatus::Pending,
        order: entities_order,
        version_number: 0,
    };
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;
    order_state = match txn.save_order(order_state).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    match txn.commit().await {
        Ok(x) => x,
        Err(_) => todo!(),
    };

    let response = instrument.exchange_client.submit_order(exchange_order).await;

    order_state.order_status = order_status_to_rest_api_order_status(response.order_status);
    order_state.update_time = current_time_millis();

    let txn = dao.begin(&mut db_connection).await;
    // TODO ignore optimistic locking exceptions, as the exchange may have
    // sent an async update (eg complete fill) before we can update here.
    match txn.update_order(order_state.clone()).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    match txn.commit().await {
        Ok(x) => x,
        Err(_) => todo!(),
    };

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(order_state.to_rest_api_order_state())
}

#[delete("/accounts/{account_key}/orders/{ext_order_id}")]
pub async fn cancel_order(dao: ThinData<Dao>,
                          access_control: ThinData<AccessControl>,
                          instrument_manager: ThinData<InstrumentManager>,
                          account_key: Path<(String,)>,
                          ext_order_id: Path<(String,)>) -> HttpResponse {
    let account_key = &account_key.0.as_str().to_string();
    let ext_order_id = &ext_order_id.0.as_str().to_string();
    let customer_key = base_api::get_customer_key();
    info!("cancel_order called for ext_order_id {ext_order_id}");
    if !access_control.is_allowed(account_key, &customer_key, Privilege::Read) {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;

    let order_state_option = match txn.get_order(account_key, ext_order_id).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };

    let mut order_state = match order_state_option {
        Some(x) => x,
        None => return HttpResponse::NotFound().finish(),
    };

    if !is_order_status_open(&order_state.order_status) {
        return HttpResponse::PreconditionFailed().finish();
    }

    order_state.order_status = OrderStatus::PendingCancel;
    match txn.update_order(order_state.clone()).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    match txn.commit().await {
        Ok(x) => x,
        Err(_) => todo!(),
    };

    let instrument = instrument_manager.get_instrument(match order_state.order.legs.first() {
        Some(x) => x,
        None => todo!(),
    }.instrument_id);
    let response = instrument.exchange_client.cancel_order(order_state.clone().order.client_order_id).await;

    order_state.order_status = order_status_to_rest_api_order_status(response.order_status);
    order_state.update_time = current_time_millis();

    let txn = dao.begin(&mut db_connection).await;
    match txn.update_order(order_state.clone()).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    match txn.commit().await {
        Ok(x) => x,
        Err(_) => todo!(),
    };

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(order_state.to_rest_api_order_state())
}
