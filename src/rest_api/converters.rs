use actix_web::web::ThinData;
use uuid::Uuid;
use crate::{entities, exchange_interface};
use crate::instrument_manager::InstrumentManager;
use crate::rest_api::trading::{Order, OrderLeg, OrderState, OrderStatus};
use crate::time::current_time_millis;

pub fn order_status_to_rest_api_order_status(order_status: exchange_interface::trading::OrderStatus)
                                             -> OrderStatus {
    match order_status {
        exchange_interface::trading::OrderStatus::Open => OrderStatus::Open,
        exchange_interface::trading::OrderStatus::Canceled => OrderStatus::Canceled,
        exchange_interface::trading::OrderStatus::Filled => OrderStatus::Filled,
        exchange_interface::trading::OrderStatus::Expired => OrderStatus::Expired,
        exchange_interface::trading::OrderStatus::Rejected => OrderStatus::Rejected,
    }
}


impl entities::trading::OrderLeg {
    pub fn to_rest_api_order_leg(&self) -> OrderLeg {
        OrderLeg {
            instrument_id: self.instrument_id,
            ratio: self.ratio,
        }
    }
}

impl entities::trading::Order {
    pub fn to_rest_api_order(&self) -> Order {
        let mut order_legs: Vec<OrderLeg> = Vec::new();
        for leg in self.legs.iter() {
            order_legs.push(leg.to_rest_api_order_leg());
        };
        Order {
            create_time: self.create_time,
            ext_order_id: Some(self.ext_order_id.clone()),
            account_key: self.account_key.clone(),
            price: self.price,
            quantity: self.quantity,
            legs: order_legs,
        }
    }
}

impl entities::trading::OrderState {
    pub fn to_rest_api_order_state(&self) -> OrderState {
        OrderState{
            update_time: self.update_time,
            order_status: self.order_status.clone(),
            remaining_quantity: 0,
            order: self.order.to_rest_api_order()
        }
    }
}

impl Order {
    pub fn to_exchange_order(&self, instrument_manager: ThinData<InstrumentManager>) -> exchange_interface::trading::Order {
        let mut order_legs: Vec<exchange_interface::trading::OrderLeg> = Vec::new();
        for leg in self.legs.iter() {
            let exchange_instrument = 
                instrument_manager.get_instrument(leg.instrument_id);
            order_legs.push(exchange_interface::trading::OrderLeg {
                instrument_id: exchange_instrument.exchange_instrument_id,
                ratio: leg.ratio,
            });
        };

        let order_exchange = exchange_interface::trading::Order {
            client_order_id: Uuid::new_v4().simple().to_string(),
            price: self.price,
            quantity: self.quantity,
            legs: order_legs,
        };
        order_exchange
    }

    pub fn to_entities_order(&self, client_order_id: String, ext_order_id: String) -> entities::trading::Order {
        let mut order_legs: Vec<entities::trading::OrderLeg> = Vec::new();

        for leg in self.legs.iter() {
            order_legs.push(entities::trading::OrderLeg {
                order_leg_id: 0,
                instrument_id: leg.instrument_id,
                ratio: leg.ratio,
            });
        };

        let order_entity = entities::trading::Order {
            order_id: 0,
            account_key: self.account_key.clone(),
            ext_order_id,
            client_order_id,
            create_time: current_time_millis(),
            price: self.price,
            quantity: self.quantity,
            legs: order_legs,
        };

        order_entity
    }
}
