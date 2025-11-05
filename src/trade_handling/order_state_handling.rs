use crate::constants::ACCOUNT_UPDATE_QUEUE_NAME;
use crate::converters::order_converters::order_status_to_rest_api_order_status;
use crate::exchange_interface::order::OrderState;
use crate::instrument_manager::InstrumentManager;
use crate::persistence::dao::Dao;
use crate::time::current_time_millis;
use crate::trade_handling::updates::AccountUpdate;
use crate::websockets::server::WebSocketServer;
use anyhow::Error;
use async_std::task;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn handle_order_state(mutex: Arc<Mutex<()>>, 
                          dao: &Dao, 
                          web_socket_server: &WebSocketServer, 
                          instrument_manager: &InstrumentManager, 
                          order_state: OrderState) {
    info!("Order state: {:?}", order_state);
    tokio::spawn(update_order_state_loop(mutex.clone(), web_socket_server.clone(), dao.clone(), instrument_manager.clone(), order_state));
}

async fn update_order_state_loop(mutex: Arc<Mutex<()>>, web_socket_server: WebSocketServer, dao: Dao, instrument_manager: InstrumentManager, order_state: OrderState) {
    let start = current_time_millis();
    let one_hundred_millis = std::time::Duration::from_millis(100);
    for attempt in 0..10 {
        match update_order_state(mutex.clone(), web_socket_server.clone(), dao.clone(), instrument_manager.clone(), &order_state).await {
            Ok(_) => {
                info!("Successfully updated order state on attempt {}", attempt);
                let end = current_time_millis();
                info!("update_order_state_loop took {} ms", end-start);
                return;
            }
            Err(update_error) => {
                warn!("Failed to update order state on attempt {}: {}", attempt, update_error);
            }
        };
        task::sleep(one_hundred_millis).await;
    }
    error!("Failed to update order state after all attempts");
}

async fn update_order_state(mutex: Arc<Mutex<()>>, 
                            mut web_socket_server: WebSocketServer, 
                            dao: Dao, 
                            instrument_manager: InstrumentManager,
                            order_state_orig: &OrderState) -> Result<(), Error> {
    let order_state = order_state_orig.clone();
    let _lock = mutex.lock().await;
    let mut db_connection = match dao.get_connection().await {
        Ok(db_connection) => db_connection,
        Err(dao_error) => {
            return Err(anyhow::anyhow!("Could not get connection: {}", dao_error.to_string()));
        },
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(txn) => txn,
        Err(dao_error) => {
            return Err(anyhow::anyhow!("Could not begin: {}", dao_error.to_string()));
        },
    };

    match txn.get_order_by_client_order_id(&order_state.order.client_order_id).await {
        Ok(db_order_state_option) => {
            match db_order_state_option {
                Some(mut db_order_state) => {
                    if db_order_state.update_time > order_state.update_time {
                        info!("Current order state has newer update time than received order state, skipping update");
                        return Ok(())
                    }
                    db_order_state.order_status = order_status_to_rest_api_order_status(order_state.order_status);
                    db_order_state.update_time = order_state.update_time;
                    match txn.update_order(&mut db_order_state).await {
                        Ok(x) => x,
                        Err(err) => {
                            return Err(anyhow::anyhow!("Unable to update order: {}", err));
                        },
                    };

                    let account_option = match txn.get_account(db_order_state.order.account_id).await {
                        Ok(account_option) => account_option,
                        Err(err) => {
                            return Err(anyhow::anyhow!("Unable to get_account: {}", err));
                        },
                    };

                    let account = match account_option {
                        Some(account) => account,
                        None => {
                            return Err(anyhow::anyhow!("No account for id: {}", db_order_state.order.account_id));
                        }
                    };

                    match txn.commit().await {
                        Ok(x) => x,
                        Err(err) => {
                            return Err(anyhow::anyhow!("Unable to commit: {}", err));
                        },
                    };
                    let rest_api_order_state = match db_order_state.to_rest_api_order_state(account.account_key.as_str(), &instrument_manager) {
                        Ok(rest_api_order_state) => rest_api_order_state,
                        Err(err) => {
                            return Err(anyhow::anyhow!("Unable to convert order_state to rest_api_order_state: {}", err));
                        },
                    };
                    let account_update = AccountUpdate {
                        balance: None,
                        position: None,
                        trade: None,
                        order_state: Some(rest_api_order_state),
                    };
                    web_socket_server.send_account_message(account.account_key.as_str(), ACCOUNT_UPDATE_QUEUE_NAME, &account_update);
                }
                _ => {
                    return Err(anyhow::anyhow!("update_order_state Trying to update unknown order {}", &order_state.order.client_order_id));
                }
            }
        },
        Err(err) => {
            return Err(anyhow::anyhow!("Unable to get_order_by_client_order_id: {}", err));
        },
    };
    Ok(())
}