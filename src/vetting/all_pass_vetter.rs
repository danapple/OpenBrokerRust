use crate::dtos;
use crate::dtos::order::VettingResult;
use crate::entities::account::Position;
use crate::entities::order::OrderState;
use std::collections::HashMap;
use std::fmt::Error;

#[derive(Clone)]
pub struct AllPassVetter {
}

impl AllPassVetter {
    pub fn new() -> AllPassVetter {
        AllPassVetter {}
    }
    pub async fn vet_order(& self,
                           rest_api_order: &dtos::order::Order,
                           viable_orders: &HashMap<String, OrderState>,
                           open_positions: &HashMap<i64, Position>) -> Result<VettingResult, Error> {
        Ok(VettingResult {
            pass: true,
            reject_reason: None
        })
    }
}
