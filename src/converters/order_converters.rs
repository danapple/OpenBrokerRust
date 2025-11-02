use crate::dtos::order::{Order, OrderLeg, OrderState, OrderStatus};
use crate::entities::account::Account;
use crate::instrument_manager::InstrumentManager;
use crate::time::current_time_millis;
use crate::{entities, exchange_interface};
use actix_web::web::ThinData;
use uuid::Uuid;

pub fn order_status_to_rest_api_order_status(order_status: exchange_interface::order::OrderStatus)
                                             -> OrderStatus {
    match order_status {
        exchange_interface::order::OrderStatus::Open => OrderStatus::Open,
        exchange_interface::order::OrderStatus::Canceled => OrderStatus::Canceled,
        exchange_interface::order::OrderStatus::Filled => OrderStatus::Filled,
        exchange_interface::order::OrderStatus::Expired => OrderStatus::Expired,
        exchange_interface::order::OrderStatus::Rejected => OrderStatus::Rejected,
    }
}

impl entities::order::OrderLeg {
    pub fn to_rest_api_order_leg(&self, instrument_manager: &InstrumentManager) -> OrderLeg {
        OrderLeg {
            instrument_key: instrument_manager.get_instrument(self.instrument_id).unwrap().unwrap().instrument_key,
            ratio: self.ratio,
        }
    }
}

impl entities::order::Order {
    pub fn to_rest_api_order(&self, account_key: &str, instrument_manager: &InstrumentManager) -> Order {
        let mut order_legs: Vec<OrderLeg> = Vec::new();
        for leg in self.legs.iter() {
            order_legs.push(leg.to_rest_api_order_leg(instrument_manager));
        };
        Order {
            create_time: self.create_time,
            order_number: Some(self.order_number),
            ext_order_id: Some(self.ext_order_id.clone()),
            account_key: Some(account_key.to_string()),
            price: self.price,
            quantity: self.quantity,
            legs: order_legs,
        }
    }
}

impl entities::order::OrderState {
    pub fn to_rest_api_order_state(&self, account_key: &str, instrument_manager: &InstrumentManager) -> OrderState {
        OrderState{
            update_time: self.update_time,
            order_status: self.order_status.clone(),
            filled_quantity: 0,
            version_number: self.version_number,
            order: self.order.to_rest_api_order(account_key, instrument_manager),
        }
    }
}

impl Order {
    pub fn to_exchange_order(&self, instrument_manager: &ThinData<InstrumentManager>) -> Result<exchange_interface::order::Order, anyhow::Error> {
        let mut order_legs: Vec<exchange_interface::order::OrderLeg> = Vec::new();
        for leg in self.legs.iter() {

            let instrument_result = instrument_manager.get_instrument_by_key(leg.instrument_key.as_str());
            let instrument_option = match instrument_result {
                Ok(instrument_option) => instrument_option,
                Err(err) => return Err(anyhow::anyhow!("Unable to get instrument: {}", err))
            };
            let exchange_instrument = match instrument_option {
                Some(instrument) => instrument,
                None => return Err(anyhow::anyhow!("No instrument with key: {}", leg.instrument_key))
            };

            order_legs.push(exchange_interface::order::OrderLeg {
                instrument_id: exchange_instrument.exchange_instrument_id,
                ratio: leg.ratio,
            });
        };

        let order_exchange = exchange_interface::order::Order {
            client_order_id: Uuid::new_v4().simple().to_string(),
            price: self.price,
            quantity: self.quantity,
            legs: order_legs,
        };
        Ok(order_exchange)
    }

    pub fn to_entities_order(&self, account: &Account, client_order_id: String, instrument_manager: &ThinData<InstrumentManager>) -> Result<entities::order::Order, anyhow::Error> {
        let mut order_legs: Vec<entities::order::OrderLeg> = Vec::new();

        for leg in self.legs.iter() {
            let instrument_option = match instrument_manager.get_instrument_by_key(&leg.instrument_key) {
                Ok(instrument_option) => instrument_option,
                Err(_) => todo!(),
            };

            let instrument = match instrument_option {
                Some(instrument) => instrument,
                None => todo!()
            };

            order_legs.push(entities::order::OrderLeg {
                order_leg_id: 0,
                instrument_id: instrument.instrument_id,
                ratio: leg.ratio,
            });
        };

        let order_entity = entities::order::Order {
            order_id: 0,
            account_id: account.account_id,
            order_number: 0,
            ext_order_id: match self.ext_order_id.clone() {
                Some(ext_order_id) => ext_order_id,
                None => return Err(anyhow::anyhow!("No external order id for account: {}", account.account_id))
            },
            client_order_id,
            create_time: current_time_millis(),
            price: self.price,
            quantity: self.quantity,
            legs: order_legs,
        };
        Ok(order_entity)
    }
}
