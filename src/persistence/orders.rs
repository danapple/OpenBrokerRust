use crate::entities::trading::{Order, OrderLeg, OrderState};
use crate::persistence::dao::{DaoError, DaoTransaction};
use crate::rest_api::trading::OrderStatus;
use log::error;
use std::collections::HashMap;
use std::str::FromStr;
use tokio_postgres::Row;


impl<'b> DaoTransaction<'b> {
    pub async fn save_order(&self, mut order_state: OrderState) -> Result<OrderState, DaoError> {

        let order_number_row = match self.transaction.query_one(
            "INSERT INTO order_number_generator \
            (accountId, lastOrderNumber) \
            VALUES ($1, 1) \
            ON CONFLICT (accountId) \
            DO UPDATE \
            SET lastOrderNumber = order_number_generator.lastOrderNumber + 1 \
            RETURNING order_number_generator.lastOrderNumber",
            &[&order_state.order.account_id,
            ]
        ).await {
            Ok(x) => x,
            Err(y) => { error!("save_order {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecuteFailed { description: y.to_string() })},
        };
        let order_number =  order_number_row.get("lastOrderNumber");

        order_state.order.order_number = order_number;

        let row = match self.transaction.query_one(
            "INSERT INTO order_base \
            (accountId, extOrderId, orderNumber, clientOrderId, createTime, price, quantity) \
            VALUES ($1, $2, $3, $4, $5, $6, $7) \
            RETURNING orderId",
            &[&order_state.order.account_id,
                     &order_state.order.ext_order_id,
                     &order_state.order.order_number,
                     &order_state.order.client_order_id,
                     &order_state.order.create_time,
                     &order_state.order.price,
                     &order_state.order.quantity,
            ]
        ).await {
            Ok(x) => x,
            Err(y) => { error!("save_order {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecuteFailed { description: y.to_string() })},
        };
        let order_id =  row.get("orderId");
        order_state.order.order_id = order_id;


        for leg in &order_state.order.legs {
            match self.transaction.query(
                "INSERT INTO order_leg \
            (orderId, instrumentId, ratio) \
            VALUES ($1, $2, $3) ",
                &[&order_id,
                    &leg.instrument_id,
                    &leg.ratio
                ]
            ).await {
                Ok(x) => x,
                Err(y) => { error!("save_order leg {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecuteFailed { description: y.to_string() })},
            };
        }

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
            Err(y) => { error!("save_order order_state {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecuteFailed { description: y.to_string() })},
        };

        match self.insert_order_state_history(&order_state).await {
            Ok(_) => {}
            Err(y) => { return Err(y) }
        }
        Ok(order_state)
    }

    pub async fn update_order(&self, order_state: &mut OrderState) -> Result<(), DaoError> {
        let next_version_number = order_state.version_number + 1;
        let rows_updated = match self.transaction.execute(
            "UPDATE order_state \
                set orderStatus = $1, updateTime = $2, versionNumber = $3 \
                WHERE orderId = $4 and versionNumber = $5",
            &[&order_state.order_status.to_string(),
                     &order_state.update_time,
                     &next_version_number,
                     &order_state.order.order_id,
                     &order_state.version_number,
            ]
        ).await {
            Ok(rows) => rows,
            Err(y) => { error!("update_order order_state {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecuteFailed { description: y.to_string() })},
        };
        if rows_updated == 0 {
            return Err(DaoError::OptimisticLockingFailed{ description: "update order 0 rows modified".to_string() });
        }
        order_state.version_number = next_version_number;
        self.insert_order_state_history(&order_state).await
    }

    async fn insert_order_state_history(&self, order_state: &OrderState) -> Result<(), DaoError> {
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
            Err(y) => { error!("save_order order_state_history {}: {}", y.to_string(), match y.as_db_error() {Some(x) => format!("{}", x),None => "none".parse().unwrap()}); return Err(DaoError::ExecuteFailed { description: y.to_string() })},
        }
    }

    pub async fn get_orders(&self, account_key: &String) -> Result<HashMap<String, OrderState>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ORDER_QUERY);
        query_string.push_str("WHERE account.accountKey = $1 ");
        query_string.push_str("ORDER BY base.orderNumber DESC");

        let res = match self.transaction.query(&query_string,
                                               &[&account_key]).await {
            Ok(x) => x,
            Err(y) => { error!("get_orders {}", y); return Err(DaoError::QueryFailed{ description: y.to_string() })},
        };

        let order_state_map = convert_rows_to_order_states(res);
        Ok(order_state_map)
    }

    pub async fn get_order_by_ext_order_id(&self, account_key: &String, ext_order_id: &String) -> Result<Option<OrderState>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ORDER_QUERY);
        query_string.push_str("WHERE account.accountKey = $1 AND base.extOrderId = $2");
       let res = match self.transaction.query(&query_string,
           &[&account_key,
                    &ext_order_id]).await {
            Ok(x) => x,
           Err(y) => { error!("get_order {}", y); return Err(DaoError::QueryFailed{ description: y.to_string() })},
        };
        let order_state_map = convert_rows_to_order_states(res);
        let order_state = match order_state_map.get(ext_order_id) {
            None => {None}
            Some(x) => { Some(x.clone()) }
        };
        Ok(order_state)
    }

    pub(crate) async fn get_order_by_client_order_id(&self, client_order_id: &String) -> Result<Option<OrderState>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ORDER_QUERY);
        query_string.push_str(" WHERE base.clientOrderId = $1");
        let res = match self.transaction.query(&query_string,
                                               &[&client_order_id]).await {
            Ok(x) => x,
            Err(y) => { error!("get_order_by_client_order_id {}", y); return Err(DaoError::QueryFailed{ description: y.to_string() })},
        };
        let order_state_map = convert_rows_to_order_states(res);
        if order_state_map.len() > 1 {
            todo!()
        }
        let order_state = order_state_map.values().next();
        let order_state = match order_state {
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
        instrument_id: row.get("instrumentId"),
        ratio: row.get("ratio"),
    };
    order_state.get_order_mut().add_leg(leg);

}

fn convert_row_to_order_state(row: &Row) -> OrderState {
    OrderState {
        order: Order {
            order_id: row.get("orderId"),
            account_id: row.get("accountId"),
            order_number: row.get("orderNumber"),
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

const ORDER_QUERY: &str = "SELECT base.orderId, base.accountId, base.orderNumber, \
base.extOrderId, base.clientOrderId, base.createTime, base.price, base.quantity, \
state.orderStatus, state.updateTime, state.versionNumber, \
leg.orderLegId, leg.instrumentId, leg.ratio \
FROM order_base AS base \
JOIN order_state AS state ON state.orderId = base.orderId \
JOIN order_leg AS leg ON leg.orderId = base.orderId \
JOIN account ON account.accountId = base.accountId ";