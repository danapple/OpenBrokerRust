use std::sync::{Arc};
use tokio::sync::Mutex;
use log::{info};
use crate::exchange_interface::trading::{Execution, OrderState, OrderStatus};
use crate::persistence::dao::Dao;
use crate::rest_api::converters::order_status_to_rest_api_order_status;

pub fn handle_order_state(mutex: Arc<Mutex<()>>, dao: &Dao, order_state: OrderState) {
    info!("Order state: {:?}", order_state);
    tokio::spawn(update_order_state(mutex.clone(), dao.clone(), order_state));
}

async fn update_order_state(mutex: Arc<Mutex<()>>, dao: Dao, order_state: OrderState) {
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
                        Err(_) => todo!(),
                    };
                },
                _ => {}
            }
        },
        Err(_) => todo!(),
    };
    match txn.commit().await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
}

pub fn handle_execution(dao: &Dao, execution: Execution) {
    info!("Execution: {:?}", execution);
    //
    // let mut db_connection = dao.get_connection().await;
    // let txn = dao.begin(&mut db_connection).await;
}
