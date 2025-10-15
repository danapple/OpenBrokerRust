use std::fmt::Error;
use crate::rest_api;
use crate::vetting::vetter::VettingResult;

#[derive(Clone)]
pub struct AllPassVetter {
}

impl AllPassVetter {
    pub fn new() -> AllPassVetter {
        AllPassVetter {}
    }
    pub async fn vet_order(& self, rest_api_order: rest_api::trading::Order) -> Result<VettingResult, Error> {
        Ok(VettingResult {
            pass: rest_api_order.quantity != 0
        })
    }
}
