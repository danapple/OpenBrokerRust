use crate::dtos;
use crate::dtos::order::VettingResult;
use crate::entities::order::OrderState;
use crate::instrument_manager::InstrumentManager;
use anyhow::Error;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Validator {
    pub instrument_manager: InstrumentManager
}

impl Validator {
    pub fn new(instrument_manager: InstrumentManager) -> Validator {
        Validator { instrument_manager }
    }
    pub fn validate_order(&self,
                          rest_api_order: &dtos::order::Order,
                          viable_orders: &HashMap<String, OrderState>) -> Result<VettingResult, Error> {
        if (rest_api_order.quantity == 0) {
            return Ok(VettingResult {
                pass: false,
                reject_reason: Some("Order quantity is 0".to_string())
            })
        }
        
        for leg in rest_api_order.legs.iter() {
            let leg_instrument_option = self.instrument_manager.get_instrument_by_key(leg.instrument_key.as_str())?;
            let leg_instrument = match leg_instrument_option {
                Some(leg_instrument) => leg_instrument,
                None => return Err(anyhow::anyhow!("Unable to find instrument for key {}", leg.instrument_key.as_str()))
            };

            for viable_order in viable_orders.values() {
                for existing_leg in viable_order.order.clone().legs {
                    if leg_instrument.instrument_id == existing_leg.instrument_id {
                        let new_leg_quantity = rest_api_order.quantity * leg.ratio;
                        let existing_leg_quantity = viable_order.order.quantity * existing_leg.ratio;

                        if (new_leg_quantity > 0 && existing_leg_quantity < 0 &&
                            rest_api_order.price >= viable_order.order.price) || (
                            new_leg_quantity < 0 && existing_leg_quantity > 0 &&
                            rest_api_order.price <= viable_order.order.price) {
                            return Ok(VettingResult {
                                pass: false,
                                reject_reason:
                                Some(format!("This order would immediately trade against your existing order # {}",
                                             viable_order.order.order_number).to_string())
                            })
                        }
                    }
                }
            }
        }
        Ok(VettingResult {
            pass: true,
            reject_reason: None
        })
    }
}
