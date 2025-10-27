use crate::persistence::dao::{Dao, DaoTransaction};
use crate::trade_handling::updates::AccountUpdate;
use crate::websockets::stomp::{parse_message, SendContent};
use log::error;
use tokio::sync::mpsc::UnboundedSender;

pub async fn send_positions(txn: DaoTransaction<'_>, conn_tx: UnboundedSender<crate::websockets::server::QueueItem>, destination: &String, account_key: &String) {
    let positions = match txn.get_positions(account_key).await {
        Ok(x) => x,
        Err(y) => {
            error!("send_positions error while getting positions: {}", y);
            return;
        },
    };
    for position in positions.values() {
        let account_update = AccountUpdate {
            position: Some(position.to_rest_api_position(account_key)),
            balance: None,
            trade: None,
            order_state: None,
        };
        let queue_item = crate::websockets::server::QueueItem {
            destination: destination.clone(),
            body: serde_json::to_string(&account_update).unwrap(),
        };
        conn_tx.send(queue_item.clone()).unwrap();
    }
}

pub async fn send_balance(txn: DaoTransaction<'_>, conn_tx: UnboundedSender<crate::websockets::server::QueueItem>, destination: &String, account_key: &String) {
    let balance = match txn.get_balance(account_key).await {
        Ok(x) => x,
        Err(y) => {
            error!("send_balance error while getting balance: {}", y);
            return;
        },
    };
    let account_update = AccountUpdate {
        position: None,
        balance: Some(balance.to_rest_api_balance(account_key)),
        trade: None,
        order_state: None,
    };
    let queue_item = crate::websockets::server::QueueItem {
        destination: destination.clone(),
        body: serde_json::to_string(&account_update).unwrap(),
    };
    conn_tx.send(queue_item.clone()).unwrap();
}

pub async fn send_orders(txn: DaoTransaction<'_>, conn_tx: UnboundedSender<crate::websockets::server::QueueItem>, destination: &String, account_key: &String) {
    let order_states = match txn.get_orders(&account_key).await {
        Ok(x) => x,
        Err(y) => {
            error!("send_orders error while getting orders: {}", y);
            return;
        },
    };
    match txn.rollback().await {
        Ok(x) => x,
        Err(y) => {
            error!("send_orders error rolling back: {}", y);
            return;
        },
    };

    for order_state in order_states.values() {
        let account_update = AccountUpdate {
            position: None,
            balance: None,
            trade: None,
            order_state: Some(order_state.to_rest_api_order_state(account_key)),
        };
        let queue_item = crate::websockets::server::QueueItem {
            destination: destination.clone(),
            body: serde_json::to_string(&account_update).unwrap(),
        };
        conn_tx.send(queue_item.clone()).unwrap();
    }
}
