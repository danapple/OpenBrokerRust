use crate::constants::ACCOUNT_UPDATE_QUEUE_NAME;
use crate::exchange_interface::trading::OrderState;
use crate::persistence::dao::Dao;
use crate::rest_api::trading_converters::order_status_to_rest_api_order_status;
use crate::trade_handling::updates::AccountUpdate;
use crate::websockets::server::WebSocketServer;
use log::{error, info};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn handle_order_state(mutex: Arc<Mutex<()>>, dao: &Dao, web_socket_server: &WebSocketServer, order_state: OrderState) {
    info!("Order state: {:?}", order_state);
    tokio::spawn(update_order_state(mutex.clone(), web_socket_server.clone(), dao.clone(), order_state));
}

async fn update_order_state(mutex: Arc<Mutex<()>>, mut web_socket_server: WebSocketServer, dao: Dao, order_state: OrderState) {
    let _lock = mutex.lock().await;
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;

    match txn.get_order_by_client_order_id(&order_state.order.client_order_id).await {
        Ok(db_order_state_option) => {
            match db_order_state_option {
                Some(mut db_order_state_option) => {
                    db_order_state_option.order_status = order_status_to_rest_api_order_status(order_state.order_status);
                    db_order_state_option.update_time = order_state.update_time;
                    match txn.update_order(&mut db_order_state_option).await {
                        Ok(x) => x,
                        Err(err) => {
                            error!("Unable to update order: {}", err);
                            return;
                        },
                    };

                    let account = match txn.get_account(db_order_state_option.order.account_id).await {
                        Ok(x) => x,
                        Err(err) => {
                            error!("Unable to get_account: {}", err);
                            return;
                        },
                    };
                    match txn.commit().await {
                        Ok(x) => x,
                        Err(err) => {
                            error!("Unable to commit: {}", err);
                            return;
                        },
                    };
                    let account_update = AccountUpdate {
                        balance: None,
                        position: None,
                        trade: None,
                        order_state: Some(db_order_state_option.to_rest_api_order_state(account.account_key.as_str())),
                    };
                    web_socket_server.send_account_message(account.account_key.as_str(), ACCOUNT_UPDATE_QUEUE_NAME, &account_update);
                },
                _ => {
                    error!("update_order_state Trying to update unknown order {}", &order_state.order.client_order_id);
                    return;
                }
            }
        },
        Err(err) => {
            error!("Unable to get_order_by_client_order_id: {}", err);
            return;
        },
    };
}