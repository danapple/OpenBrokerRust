use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Index;
use std::str::FromStr;
use crate::entities::trading::{Order, OrderLeg, OrderState};
use deadpool_postgres::{Pool, Transaction};
use log::error;
use tokio_postgres::{Row};
use crate::rest_api::trading::OrderStatus;
use crate::persistence::dao::{DaoError, DaoTransaction};


impl<'b> DaoTransaction<'b> {
    pub async fn save_order(&self, mut order_state: OrderState) -> Result<OrderState, DaoError> {
        let row = match self.transaction.query_one(
            "INSERT INTO order_base \
            (accountKey, extOrderId, clientOrderId, createTime, price, quantity) \
            VALUES ($1, $2, $3, $4, $5, $6) \
            RETURNING orderId",
            &[&order_state.order.account_key,
                     &order_state.order.ext_order_id,
                     &order_state.order.client_order_id,
                     &order_state.order.create_time,
                     &order_state.order.price,
                     &order_state.order.quantity,
            ]
        ).await {
            Ok(x) => x,
            Err(y) => { error!("save_order {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecutedFailed{ description: y.to_string() })},
        };
        let order_id =  row.get("orderId");
        order_state.order.order_id = order_id;

        match self.transaction.execute(
            "INSERT INTO order_state \
                (orderId, orderStatus, updateTime, versionNumber) \
                VALUES ($1, $2, $3, $4)",
            &[&order_state.order.order_id,
                     &order_state.order_status.to_string(),
                     &order_state.update_time,
                     &order_state.version_number
            ]
        ).await {
            Ok(_) => 0,
            Err(y) => { error!("save_order order_state {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecutedFailed{ description: y.to_string() })},
        };

        match self.insert_order_state_history(&order_state).await {
            Ok(_) => {}
            Err(y) => { return Err(y) }
        }
        Ok(order_state)
    }

    pub async fn update_order(&self, order_state: OrderState) -> Result<(), DaoError> {
        let rows_updated = match self.transaction.execute(
            "UPDATE order_state \
                set orderStatus = $2, updateTime = $3, versionNumber = $4 + 1 \
                WHERE orderId = $1 and versionNumber = $4",
            &[&order_state.order.order_id,
                     &order_state.order_status.to_string(),
                     &order_state.update_time,
                     &order_state.version_number
            ]
        ).await {
            Ok(_) => 0,
            Err(y) => { error!("update_order order_state {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecutedFailed{ description: y.to_string() })},
        };
        if rows_updated == 0 {
            return Err(DaoError::OptimisticLockingFailed{ description: "update order".to_string() });
        }
        self.insert_order_state_history(&order_state).await
    }

    pub async fn insert_order_state_history(&self, order_state: &OrderState) -> Result<(), DaoError> {
        match self.transaction.execute(
            "INSERT INTO order_state_history \
                (orderId, orderStatus, createTime, versionNumber) \
                VALUES ($1, $2, $3, $4)",
            &[&order_state.order.order_id,
                     &order_state.order_status.to_string(),
                     &order_state.update_time,
                     &order_state.version_number
            ]
        ).await {
            Ok(_) => Ok(()),
            Err(y) => { error!("save_order order_state_history {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecutedFailed{ description: y.to_string() })},
        }
    }

    pub async fn get_orders(&self, account_key: &String) -> Result<HashMap<String, OrderState>, DaoError> {
        let res = match self.transaction.query(ORDER_QUERY,
                                               &[&account_key]).await {
            Ok(x) => x,
            Err(y) => { error!("get_order {}", y); return Err(DaoError::QueryFailed{ description: y.to_string() })},
        };

        let order_state_map = convert_rows_to_order_states(res);
        Ok(order_state_map)
    }

    pub async fn get_order(&self, account_key: &String, ext_order_id: &String) -> Result<Option<OrderState>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ORDER_QUERY);
        query_string.push_str(" AND base.extOrderId = $2");
       let res = match self.transaction.query(&query_string,
           &[&account_key,
                    &ext_order_id]).await {
            Ok(x) => x,
           Err(y) => { error!("get_order {}", y); return Err(DaoError::QueryFailed{ description: y.to_string() })},
        };
        let order_state_map = convert_rows_to_order_states(res);
        let order_state = match order_state_map.get(ext_order_id) {
            None => {None}
            Some(x) => {Some(x.clone())}
        };
        Ok(order_state)
    }
}

fn convert_rows_to_order_states(res: Vec<Row>) -> HashMap<String, OrderState> {
    let mut order_states = HashMap::new();
    for row in res {
        let ext_order_id: String = row.get("extOrderId");
        let order_state_entry = order_states.entry(ext_order_id);
        let order_state = order_state_entry.or_insert_with(|| convert_row_to_order_state(&row));
        add_leg_to_order_state(order_state, &row);
    }

    order_states
}

fn add_leg_to_order_state(order_state: &mut OrderState, row: &Row) {
    let leg = OrderLeg {
        order_leg_id: row.get("orderLegId"),
        instrument_id: row.get("instrument_id"),
        ratio: row.get("ratio"),
    };
    order_state.get_order_mut().add_leg(leg);

}

fn convert_row_to_order_state(row: &Row) -> OrderState {
    OrderState {
        order: Order {
            order_id: row.get("orderId"),
            account_key: row.get("accountKey"),
            ext_order_id: row.get("extOrderId"),
            client_order_id: row.get("clientOrderId"),
            create_time: row.get("createTime"),
            price: row.get("price"),
            quantity: row.get("quantity"),
            legs: vec![],
        },
        update_time: row.get("updateTime"),
        order_status: OrderStatus::from_str(row.get("orderStatus")).unwrap(),
        version_number: row.get("versionNumber"),
    }
}

const ORDER_QUERY: &str = "SELECT base.orderId, base.accountKey, base.extOrderId, base.clientOrderId, base.createTime, base.price, base.quantity, \
state.orderStatus, state.updateTime, state.versionNumber, \
leg.orderLegId, leg.instrumentId, leg.ratio \
FROM order_base AS base \
JOIN order_state AS state ON state.orderId = base.orderId \
JOIN order_leg AS leg ON leg.orderId = base.orderId \
WHERE base.accountKey = $1";