use crate::constants::ACCOUNT_UPDATE_QUEUE_NAME;
use crate::converters::order_converters::order_status_to_rest_api_order_status;
use crate::exchange_interface::order::OrderState;
use crate::instrument_manager::InstrumentManager;
use crate::persistence::dao::Dao;
use crate::trade_handling::updates::AccountUpdate;
use crate::websockets::server::WebSocketServer;
use log::{error, info};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn handle_order_state(mutex: Arc<Mutex<()>>, dao: &Dao, web_socket_server: &WebSocketServer, instrument_manager: &InstrumentManager, order_state: OrderState) {
    info!("Order state: {:?}", order_state);
    tokio::spawn(update_order_state(mutex.clone(), web_socket_server.clone(), dao.clone(), instrument_manager.clone(), order_state));
}

async fn update_order_state(mutex: Arc<Mutex<()>>, mut web_socket_server: WebSocketServer, dao: Dao, instrument_manager: InstrumentManager, order_state: OrderState) {
    let _lock = mutex.lock().await;
    let mut db_connection = match dao.get_connection().await {
        Ok(db_connection) => db_connection,
        Err(dao_error) => {
            error!("Could not get connection: {}", dao_error.to_string());
            return;
        },
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(txn) => txn,
        Err(dao_error) => {
            error!("Could not begin: {}", dao_error.to_string());
            return;
        },
    };

    match txn.get_order_by_client_order_id(&order_state.order.client_order_id).await {
        Ok(db_order_state_option) => {
            match db_order_state_option {
                Some(mut db_order_state) => {
                    db_order_state.order_status = order_status_to_rest_api_order_status(order_state.order_status);
                    db_order_state.update_time = order_state.update_time;
                    match txn.update_order(&mut db_order_state).await {
                        Ok(x) => x,
                        Err(err) => {
                            error!("Unable to update order: {}", err);
                            return;
                        },
                    };

                    let account_option = match txn.get_account(db_order_state.order.account_id).await {
                        Ok(account_option) => account_option,
                        Err(err) => {
                            error!("Unable to get_account: {}", err);
                            return;
                        },
                    };

                    let account = match account_option {
                        Some(account) => account,
                        None => {
                            error!("No account for id: {}", db_order_state.order.account_id);
                            return;
                        }
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
                        order_state: Some(db_order_state.to_rest_api_order_state(account.account_key.as_str(), &instrument_manager)),
                    };
                    web_socket_server.send_account_message(account.account_key.as_str(), ACCOUNT_UPDATE_QUEUE_NAME, &account_update);
                }
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