use crate::persistence::dao::DaoTransaction;
use crate::trade_handling::updates::AccountUpdate;
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
        let body = match serde_json::to_string(&account_update) {
            Ok(body) => body,
            Err(json_error) => {
                error!("send_positions error while serializing: {}", json_error);
                return;
            },
        };
        let queue_item = crate::websockets::server::QueueItem {
            destination: destination.clone(),
            body,
        };
        match conn_tx.send(queue_item.clone()) {
            Ok(_) => {},
            Err(y) => {
                error!("send_positions error while sending: {}", y);
                return;
            },
        };
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
    let body = match serde_json::to_string(&account_update) {
        Ok(body) => body,
        Err(json_error) => {
            error!("send_balance error while serializing: {}", json_error);
            return;
        },
    };
    let queue_item = crate::websockets::server::QueueItem {
        destination: destination.clone(),
        body,
    };
    match conn_tx.send(queue_item.clone()) {
        Ok(_) => {},
        Err(y) => {
            error!("send_balance error while sending: {}", y);
            return;
        },
    };
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
        let body = match serde_json::to_string(&account_update) {
            Ok(body) => body,
            Err(json_error) => {
                error!("send_orders error while serializing: {}", json_error);
                return;
            },
        };
        let queue_item = crate::websockets::server::QueueItem {
            destination: destination.clone(),
            body
        };
        match conn_tx.send(queue_item.clone()) {
            Ok(_) => {},
            Err(y) => {
                error!("send_orders error while sending: {}", y);
                return;
            },
        };
    }
}
