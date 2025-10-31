use crate::entities::trading::{Order, OrderLeg, OrderState};
use crate::persistence::dao::{gen_dao_error, DaoError, DaoTransaction};
use crate::rest_api::trading::{is_order_status_open, OrderStatus};
use crate::time::current_time_millis;
use std::collections::HashMap;
use std::str::FromStr;
use strum::IntoEnumIterator;
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
            Err(db_error) => { return Err(gen_dao_error("save_order order number", db_error)); }
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
            Err(db_error) => { return Err(gen_dao_error("save_order order_base", db_error)); }
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
                Err(db_error) => { return Err(gen_dao_error("save_order order_leg", db_error)); }
            };
        }

        let order_state_row_count = match self.transaction.execute(
            "INSERT INTO order_state \
                (orderId, orderStatus, updateTime, versionNumber) \
                VALUES ($1, $2, $3, $4)",
            &[&order_state.order.order_id,
                     &order_state.order_status.to_string(),
                     &order_state.update_time,
                     &order_state.version_number
            ]
        ).await {
            Ok(row_count) => row_count,
            Err(db_error) => { return Err(gen_dao_error("save_order order_state", db_error)); }
        };
        if order_state_row_count != 1 {
            return Err(DaoError::ExecuteFailed { description: format!("save_order order_state insert returned {} rows, not 1", order_state_row_count) });
        }
        let order_state_history_row_count = match self.insert_order_state_history(&order_state).await {
            Ok(row_count) => row_count,
            Err(db_error) => { return Err(db_error) }
        };
        if order_state_history_row_count != 1 {
            return Err(DaoError::ExecuteFailed { description: format!("save_order order_state_history insert returned {} rows, not 1", order_state_row_count) });
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
            Err(db_error) => { return Err(gen_dao_error("update_order order_state", db_error)); }

        };
        if rows_updated == 0 {
            return Err(DaoError::OptimisticLockingFailed{ description: "update order 0 rows modified".to_string() });
        }
        order_state.version_number = next_version_number;
        let order_state_history_row_count = match self.insert_order_state_history(&order_state).await {
            Ok(order_state_history_row_count) => order_state_history_row_count,
            Err(db_error) => return Err(db_error)
        };

        if order_state_history_row_count != 1 {
            return Err(DaoError::ExecuteFailed { description: format!("update_order order_state_history insert returned {} rows, not 1", order_state_history_row_count) });
        };
        Ok(())
    }

    async fn insert_order_state_history(&self, order_state: &OrderState) -> Result<u64, DaoError> {
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
            Ok(row_count) => Ok(row_count),
            Err(db_error) => { return Err(gen_dao_error("save_order order_state_history", db_error)); }

        }
    }

    pub async fn get_orders(&self, account_key: &String) -> Result<HashMap<String, OrderState>, DaoError> {
        let mut open_statuses = Vec::new();
        for order_status in OrderStatus::iter() {
            if (is_order_status_open(&order_status)) {
                open_statuses.push(order_status.to_string());
            }
        }

        let mut query_string: String = "".to_owned();
        query_string.push_str(ORDER_QUERY);
        query_string.push_str("WHERE account.accountKey = $1 ");
        query_string.push_str(" AND (state.updateTime > $2 OR state.orderStatus = ANY ($3)) ");
        query_string.push_str("ORDER BY base.orderNumber DESC");

        let current_time_millis = current_time_millis();
        let day_ago = current_time_millis - 86400 * 1000;
        let res = match self.transaction.query(&query_string,
                                               &[&account_key, &day_ago, &open_statuses]).await {
            Ok(x) => x,
            Err(db_error) => { return Err(gen_dao_error("get_orders", db_error)); }

        };

        let order_state_map_result = convert_rows_to_order_states(res);
        match order_state_map_result {
            Ok(order_state_map) => Ok(order_state_map),
            Err(db_error) => Err(db_error)
        }
    }

    pub async fn get_order_by_ext_order_id(&self, account_key: &String, ext_order_id: &String) -> Result<Option<OrderState>, DaoError> {
        let mut query_string: String = "".to_owned();
        query_string.push_str(ORDER_QUERY);
        query_string.push_str("WHERE account.accountKey = $1 AND base.extOrderId = $2");
       let res = match self.transaction.query(&query_string,
           &[&account_key,
                    &ext_order_id]).await {
            Ok(x) => x,
           Err(db_error) => { return Err(gen_dao_error("get_order", db_error)); }
       };
        if res.len() > 1 {
            if res.len() > 1 {
                return Err(DaoError::QueryFailed {
                    description: format!("get_order_by_ext_order_id got {} rows, expected 1", res.len()),
                });
            }
        }
        let order_state_map_result = convert_rows_to_order_states(res);
        let order_state_map = match order_state_map_result {
            Ok(order_state_map) => order_state_map,
            Err(db_error) => return Err(db_error)
        };

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
            Err(db_error) => { return Err(gen_dao_error("get_order_by_client_order_id", db_error)); }
        };
        if res.len() > 1 {
            if res.len() > 1 {
                return Err(DaoError::QueryFailed {
                    description: format!("get_order_by_client_order_id got {} rows, expected 1", res.len()),
                });
            }
        }
        let order_state_map_result = convert_rows_to_order_states(res);
        let order_state_map = match order_state_map_result {
            Ok(order_state_map) => order_state_map,
            Err(db_error) => return Err(db_error)
        };
        let order_state = order_state_map.values().next();
        let order_state = match order_state {
            None => {None}
            Some(x) => {Some(x.clone())}
        };
        Ok(order_state)
    }
}

fn convert_rows_to_order_states(res: Vec<Row>) -> Result<HashMap<String, OrderState>, DaoError> {
    let mut order_states = HashMap::new();
    for row in res {
        let ext_order_id: String = row.get("extOrderId");
        let order_state_result = order_states.get_mut(&ext_order_id);
        match order_state_result {
            Some(order_state) => {
                add_leg_to_order_state(order_state, &row);
            }
            None => {
                let mut order_state = match convert_row_to_order_state(&row) {
                    Ok(order_state) => order_state,
                    Err(dao_error) => return Err(dao_error),
                };
                add_leg_to_order_state(&mut order_state, &row);
                order_states.insert(ext_order_id, order_state);
            }
        }
    }
    Ok(order_states)
}

fn add_leg_to_order_state(order_state: &mut OrderState, row: &Row) {
    let leg = OrderLeg {
        order_leg_id: row.get("orderLegId"),
        instrument_id: row.get("instrumentId"),
        ratio: row.get("ratio"),
    };
    order_state.get_order_mut().add_leg(leg);
}

fn convert_row_to_order_state(row: &Row) -> Result<OrderState, DaoError> {
    let row_order_status = row.get("orderStatus");
    let order_status_result = OrderStatus::from_str(row_order_status);
    let order_status = match order_status_result {
        Ok(order_status) => order_status,
        Err(()) => {
            return Err(DaoError::ConversionFailed {
                description: format!("Unknown order status {}", row_order_status)
            })
        }
    };
    Ok(OrderState {
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
        order_status,
        version_number: row.get("versionNumber"),
    })
}

const ORDER_QUERY: &str = "SELECT base.orderId, base.accountId, base.orderNumber, \
base.extOrderId, base.clientOrderId, base.createTime, base.price, base.quantity, \
state.orderStatus, state.updateTime, state.versionNumber, \
leg.orderLegId, leg.instrumentId, leg.ratio \
FROM order_base AS base \
JOIN order_state AS state ON state.orderId = base.orderId \
JOIN order_leg AS leg ON leg.orderId = base.orderId \
JOIN account ON account.accountId = base.accountId ";