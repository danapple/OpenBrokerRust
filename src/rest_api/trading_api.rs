use crate::access_control::{AccessControl, Privilege};
use std::collections::HashMap;
use std::string::ToString;

use crate::constants::{ACCOUNT_UPDATE_QUEUE_NAME, APPLICATION_JSON};
use actix_web::web::{Json, Path, ThinData};
use actix_web::{web, HttpRequest, HttpResponse};
use log::{error, info};
use uuid::Uuid;

use crate::instrument_manager::InstrumentManager;
use crate::persistence::dao::Dao;
use crate::rest_api::base_api;
use crate::rest_api::base_api::send_order_state;
use crate::rest_api::trading::{is_order_status_open, Order, OrderState, OrderStatus, VettingResult};
use crate::rest_api::trading_converters::order_status_to_rest_api_order_status;
use crate::time::current_time_millis;
use crate::vetting::all_pass_vetter::AllPassVetter;
use crate::websockets::server::WebSocketServer;
use crate::{entities, exchange_interface};

#[get("/accounts/{account_key}/orders")]
pub async fn get_orders(dao: ThinData<Dao>,
                        access_control: ThinData<AccessControl>,
                        path: Path<(String)>,
                        req: HttpRequest,
) -> HttpResponse {
    info!("get_orders called");
    let account_key = path.into_inner();

    let customer_key = base_api::get_customer_key(req);
    if !access_control.is_allowed(&account_key, customer_key, Privilege::Read).await {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;

    let order_states = match txn.get_orders(&account_key).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    match txn.rollback().await {
        Ok(x) => x,
        Err(_) => todo!(),
    };

    let mut api_order_states: HashMap<String, OrderState> = HashMap::new();
    for order_state in order_states {
        api_order_states.insert(order_state.0, order_state.1.to_rest_api_order_state(account_key.as_str()));
    }
    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(api_order_states)
}

#[get("/accounts/{account_key}/orders/{ext_order_id}")]
pub async fn get_order(dao: ThinData<Dao>,
                       access_control: ThinData<AccessControl>,
                       path: Path<(String, String)>,
                       req: HttpRequest,) -> HttpResponse {
    let (account_key, ext_order_id) = path.into_inner();
    let customer_key = base_api::get_customer_key(req);
    info!("get_order called for ext_order_id {ext_order_id}");
    if !access_control.is_allowed(&account_key, customer_key, Privilege::Read).await {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;

    let order_state_option = match txn.get_order_by_ext_order_id(&account_key, &ext_order_id).await {
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
            .json(x.to_rest_api_order_state(account_key.as_str())),
        None => HttpResponse::NotFound().finish()
    }
}


#[post("/accounts/{account_key}/previewOrder")]
pub async fn preview_order(dao: ThinData<Dao>,
                           access_control: ThinData<AccessControl>,
                           instrument_manager: ThinData<InstrumentManager>,
                           vetter: ThinData<AllPassVetter>,
                           path: Path<(String)>,
                           req: HttpRequest,
                           rest_api_order: Json<Order>) -> HttpResponse {
    let account_key = path.into_inner();
    let customer_key = base_api::get_customer_key(req);
    if !access_control.is_allowed(&account_key, customer_key, Privilege::Read).await {
        return HttpResponse::Forbidden().finish();
    }

    let vetting_result = match vetter.vet_order(&rest_api_order).await {
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
                          mut web_socket_server: ThinData<WebSocketServer>,
                          path: Path<(String)>,
                          req: HttpRequest,
                          mut rest_api_order: Json<Order>) -> HttpResponse {
    info!("submit_order called");
    let account_key = path.into_inner();

    let customer_key = base_api::get_customer_key(req);
    if !access_control.is_allowed(&account_key, customer_key, Privilege::Read).await {
        return HttpResponse::Forbidden().finish();
    }

    let vetting_result = match vetter.vet_order(&rest_api_order).await {
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

    rest_api_order.account_key = Some(account_key.clone());
    rest_api_order.ext_order_id = Some(ext_order_id);

    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;
    let account = match txn.get_account_by_account_key(&account_key.clone()).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    let entities_order = rest_api_order.to_entities_order(&account, exchange_order.client_order_id.clone());
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
    send_order_state(&mut web_socket_server, &account_key, &order_state);

    let response = instrument.exchange_client.submit_order(exchange_order).await;

    // We'll get async notifications for all status updates other than Rejected
    if response.order_status == exchange_interface::trading::OrderStatus::Rejected {
        order_state.order_status = OrderStatus::Rejected;
        order_state.update_time = current_time_millis();

        let txn = dao.begin(&mut db_connection).await;
        match txn.update_order(&mut order_state).await {
            Ok(x) => x,
            Err(y) => {
                error!("update error {}", y.to_string());
                todo!()
            },
        };
        match txn.commit().await {
            Ok(x) => x,
            Err(_) => todo!(),
        };
        send_order_state(&mut web_socket_server, &account_key, &order_state);
    }

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(order_state.to_rest_api_order_state(account_key.as_str()))
}

#[delete("/accounts/{account_key}/orders/{ext_order_id}")]
pub async fn cancel_order(dao: ThinData<Dao>,
                          mut web_socket_server: ThinData<WebSocketServer>,
                          access_control: ThinData<AccessControl>,
                          instrument_manager: ThinData<InstrumentManager>,
                          path: Path<(String, String)>,
                          req: HttpRequest,) -> HttpResponse {
    let (account_key, ext_order_id) = path.into_inner();

    let customer_key = base_api::get_customer_key(req);
    info!("cancel_order called for ext_order_id {ext_order_id}");
    if !access_control.is_allowed(&account_key, customer_key, Privilege::Read).await {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;

    let order_state_option = match txn.get_order_by_ext_order_id(&account_key, &ext_order_id).await {
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
    match txn.update_order(&mut order_state).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    match txn.commit().await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    send_order_state(&mut web_socket_server, &account_key, &order_state);

    let instrument = instrument_manager.get_instrument(match order_state.order.legs.first() {
        Some(x) => x,
        None => todo!(),
    }.instrument_id);
    let response = instrument.exchange_client.cancel_order(order_state.clone().order.client_order_id).await;

    order_state.order_status = order_status_to_rest_api_order_status(response.order_status);
    order_state.update_time = current_time_millis();

    let txn = dao.begin(&mut db_connection).await;
    match txn.update_order(&mut order_state).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    match txn.commit().await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    send_order_state(&mut web_socket_server, &account_key, &order_state);

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(order_state.to_rest_api_order_state(account_key.as_str()))
}
