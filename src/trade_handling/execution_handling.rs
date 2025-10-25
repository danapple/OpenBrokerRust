use crate::constants::ACCOUNT_UPDATE_QUEUE_NAME;
use crate::entities::account::Position;
use crate::exchange_interface::trading::Execution;
use crate::instrument_manager::InstrumentManager;
use crate::persistence::dao::Dao;
use crate::time::current_time_millis;
use crate::trade_handling::updates::AccountUpdate;
use crate::websockets::server::WebSocketServer;
use log::{error, info};
use std::ops::Neg;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn handle_execution(mutex: Arc<Mutex<()>>, dao: &Dao, web_socket_server: &WebSocketServer, instrument_manager: InstrumentManager, execution: Execution) {
    info!("Execution: {:?}", execution);
    tokio::spawn(handle_execution_thread(mutex.clone(), web_socket_server.clone(), dao.clone(), instrument_manager, execution));
}

async fn handle_execution_thread(mutex: Arc<Mutex<()>>, mut web_socket_server: WebSocketServer, dao: Dao, instrument_manager: InstrumentManager, execution: Execution) {
    let _lock = mutex.lock().await;
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;
    let db_order_state_option = match txn.get_order_by_client_order_id(&execution.client_order_id).await {
        Err(err) => {
            error!("Unable to get_order_by_client_order_id: {}", err);
            return;
        },
        Ok(db_order_state_option) => {
            db_order_state_option
        }
    };

    let db_order_state = match db_order_state_option {
        Some(db_order_state) => {
            db_order_state
        }
        None => {
            error!("handle_execution_thread Trying to update unknown order {}", &execution.client_order_id);
            return;
        }
    };

    let account = match txn.get_account(db_order_state.order.account_id).await {
        Ok(x) => x,
        Err(err) => {
            error!("Unable to get_account: {}", err);
            return;
        },
    };

    let execution_cost = (execution.price * execution.quantity as f32);

    let mut balance = match txn.get_balance(&account.account_key).await {
        Ok(x) => x,
        Err(err) => {
            error!("Unable to get_balance: {}", err);
            return;
        },
    };

    balance.cash -= execution_cost;
    balance.update_time = current_time_millis();

    match txn.update_balance(&mut balance).await {
        Ok(_) => {},
        Err(err) => {
            error!("Unable to update_balance: {}", err);
            return;
        },
    };

    let instrument = instrument_manager.get_instrument_by_exchange_instrument_id(execution.instrument_id);

    let position_result = txn.get_position(&account.account_key, instrument.instrument_id).await;

    let position_option = match position_result {
        Ok(x) => x,
        Err(err) => {
            error!("Unable to get_position: {}", err);
            return;
        },
    };
    let mut position = match position_option {
        Some(position) => position,
        None => {
            let new_position = Position {
                position_id: 0,
                account_id: account.account_id,
                instrument_id: execution.instrument_id,
                quantity: 0,
                cost: 0.0,
                closed_gain: 0.0,
                update_time: current_time_millis(),
                version_number: 0,
            };
            match txn.save_position(new_position).await {
                Ok(x) => x,
                Err(err) => {
                    error!("Unable to save_position: {}", err);
                    return;
                },
            }
        }
    };
    apply_execution(&mut position, execution);

    match txn.update_position(&mut position).await {
        Ok(_) => {},
        Err(err) => {
            error!("Unable to update_balance: {}", err);
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
        balance: Some(balance.to_rest_api_balance(account.account_key.as_str())),
        position: Some(position.to_rest_api_position(account.account_key.as_str())),
        trade: None,
        order_state: None,
    };
    web_socket_server.send_account_message(account.account_key.as_str(), ACCOUNT_UPDATE_QUEUE_NAME, &account_update);

}

fn apply_execution(position: &mut Position, execution: Execution) {
    let mut opening_quantity = execution.quantity;

    // closing
    if position.quantity != 0 && execution.quantity.signum() != position.quantity.signum() {
        let mut closing_quantity = execution.quantity;
        let excess = execution.quantity.abs() - position.quantity.abs();
        if excess > 0 {
            closing_quantity -= excess * execution.quantity.signum();
            opening_quantity = excess * execution.quantity.signum();
        } else {
            opening_quantity = 0;
        }

        let cost_basis = position.cost / position.quantity as f32;
        let closing_cost = closing_quantity as f32 * cost_basis;

        position.quantity += closing_quantity;
        position.cost += closing_cost;
        position.closed_gain += closing_quantity.neg() as f32 * (execution.price - cost_basis);
        position.update_time = current_time_millis();

        if position.quantity == 0 {
            position.cost = 0f32;   // Clear out cumulative rounding errors
        }
    }
    position.quantity += opening_quantity;
    position.cost += opening_quantity as f32 * execution.price;
}


#[cfg(test)]
mod tests {
    use crate::entities::account::Position;
    use crate::exchange_interface::trading::Execution;
    use crate::time::current_time_millis;
    use crate::trade_handling::execution_handling::apply_execution;

    #[test]
    async fn test_flat_position_empty_execution() {
        let mut position = Position {
            position_id: 0,
            account_id: 0,
            instrument_id: 0,
            quantity: 0,
            cost: 0.0,
            closed_gain: 0.0,
            update_time: current_time_millis(),
            version_number: 0,
        };
        let execution = Execution {
            client_order_id: "".to_string(),
            instrument_id: 0,
            create_time: 0,
            price: 0.0,
            quantity: 0,
        };
        apply_execution(&mut position, execution);
        assert_eq!(position.quantity, 0);
        assert_eq!(round(position.cost, 2), round(0f32, 2));
        assert_eq!(round(position.closed_gain, 2), round(0.0, 2));
    }

    #[test]
    async fn test_flat_position_buy_execution() {
        let mut position = Position {
            position_id: 0,
            account_id: 0,
            instrument_id: 0,
            quantity: 0,
            cost: 0.0,
            closed_gain: 0.0,
            update_time: current_time_millis(),
            version_number: 0,
        };
        let execution = Execution {
            client_order_id: "".to_string(),
            instrument_id: 0,
            create_time: 0,
            price: 3.3,
            quantity: 3,
        };
        apply_execution(&mut position, execution);
        assert_eq!(position.quantity, 3);
        assert_eq!(round(position.cost, 2), round(3.3 * 3f32, 2));
        assert_eq!(round(position.closed_gain, 2), round(0.0, 2));
    }

    #[test]
    async fn test_flat_position_sell_execution() {
        let mut position = Position {
            position_id: 0,
            account_id: 0,
            instrument_id: 0,
            quantity: 0,
            cost: 0.0,
            closed_gain: 0.0,
            update_time: current_time_millis(),
            version_number: 0,
        };
        let execution = Execution {
            client_order_id: "".to_string(),
            instrument_id: 0,
            create_time: 0,
            price: 3.2,
            quantity: -7,
        };
        apply_execution(&mut position, execution);
        assert_eq!(position.quantity, -7);
        assert_eq!(round(position.cost, 2), round(3.2 * -7f32, 2));
        assert_eq!(round(position.closed_gain, 2), round(0.0, 2));
    }

    #[test]
    async fn test_long_position_buy_execution() {
        let mut position = Position {
            position_id: 0,
            account_id: 0,
            instrument_id: 0,
            quantity: 5,
            cost: 20f32,
            closed_gain: 0.0,
            update_time: current_time_millis(),
            version_number: 0,
        };
        let execution = Execution {
            client_order_id: "".to_string(),
            instrument_id: 0,
            create_time: 0,
            price: 30f32,
            quantity: 2,
        };
        apply_execution(&mut position, execution);
        assert_eq!(position.quantity, 7);
        assert_eq!(round(position.cost, 2), round(80f32, 2));
        assert_eq!(round(position.closed_gain, 2), round(0.0, 2));
    }

    #[test]
    async fn test_long_position_sell_execution() {
        let mut position = Position {
            position_id: 0,
            account_id: 0,
            instrument_id: 0,
            quantity: 7,
            cost: 70f32,
            closed_gain: 0.0,
            update_time: current_time_millis(),
            version_number: 0,
        };
        let execution = Execution {
            client_order_id: "".to_string(),
            instrument_id: 0,
            create_time: 0,
            price: 10f32,
            quantity: -2,
        };
        apply_execution(&mut position, execution);
        assert_eq!(position.quantity, 5);
        assert_eq!(round(position.cost, 2), round(50f32, 2));
        assert_eq!(round(position.closed_gain, 2), round(0.0, 2));
    }

    #[test]
    async fn test_short_position_buy_execution() {
        let mut position = Position {
            position_id: 0,
            account_id: 0,
            instrument_id: 0,
            quantity: -5,
            cost: -50f32,
            closed_gain: 0.0,
            update_time: current_time_millis(),
            version_number: 0,
        };
        let execution = Execution {
            client_order_id: "".to_string(),
            instrument_id: 0,
            create_time: 0,
            price: 10f32,
            quantity: 2,
        };
        apply_execution(&mut position, execution);
        assert_eq!(position.quantity, -3);
        assert_eq!(round(position.cost, 2), round(-30f32, 2));
        assert_eq!(round(position.closed_gain, 2), round(0.0, 2));
    }

    #[test]
    async fn test_short_position_sell_execution() {
        let mut position = Position {
            position_id: 0,
            account_id: 0,
            instrument_id: 0,
            quantity: -9,
            cost: -90f32,
            closed_gain: 0.0,
            update_time: current_time_millis(),
            version_number: 0,
        };
        let execution = Execution {
            client_order_id: "".to_string(),
            instrument_id: 0,
            create_time: 0,
            price: 20f32,
            quantity: -2,
        };
        apply_execution(&mut position, execution);
        assert_eq!(position.quantity, -11);
        assert_eq!(round(position.cost, 2), round(-130f32, 2));
        assert_eq!(round(position.closed_gain, 2), round(0.0, 2));
    }

    #[test]
    async fn test_long_position_oversell_execution() {
        let mut position = Position {
            position_id: 0,
            account_id: 0,
            instrument_id: 0,
            quantity: 7,
            cost: 70f32,
            closed_gain: 0.0,
            update_time: current_time_millis(),
            version_number: 0,
        };
        let execution = Execution {
            client_order_id: "".to_string(),
            instrument_id: 0,
            create_time: 0,
            price: 12f32,
            quantity: -9,
        };
        apply_execution(&mut position, execution);
        assert_eq!(position.quantity, -2);
        assert_eq!(round(position.cost, 2), round(-24f32, 2));
        assert_eq!(round(position.closed_gain, 2), round(14f32, 2));
    }

    #[test]
    async fn test_short_position_overbuy_execution() {
        let mut position = Position {
            position_id: 0,
            account_id: 0,
            instrument_id: 0,
            quantity: -5,
            cost: -50f32,
            closed_gain: 0.0,
            update_time: current_time_millis(),
            version_number: 0,
        };
        let execution = Execution {
            client_order_id: "".to_string(),
            instrument_id: 0,
            create_time: 0,
            price: 12f32,
            quantity: 9,
        };
        apply_execution(&mut position, execution);
        assert_eq!(position.quantity, 4);
        assert_eq!(round(position.cost, 2), round(48f32, 2));
        assert_eq!(round(position.closed_gain, 2), round(-10f32, 2));
    }

    fn round(val: f32, digits: u32) -> f32{
        let power = 10_i8.pow(digits) as u8;
        let pw = power as f32;
        let big_val = val * pw;
        let truncated_val = big_val as i64;
        let rounded_val = truncated_val as f32 / pw ;
        println!("rounded_val = {}", rounded_val);
        rounded_val
    }
}
