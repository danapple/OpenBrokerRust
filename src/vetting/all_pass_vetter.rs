use crate::dtos;
use crate::vetting::vetter::VettingResult;
use std::fmt::Error;

#[derive(Clone)]
pub struct AllPassVetter {
}

impl AllPassVetter {
    pub fn new() -> AllPassVetter {
        AllPassVetter {}
    }
    pub async fn vet_order(& self, 
                           rest_api_order: &dtos::order::Order) -> Result<VettingResult, Error> {
        Ok(VettingResult {
            pass: rest_api_order.quantity != 0
        })
    }
}
