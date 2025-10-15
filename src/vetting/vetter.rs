use crate::rest_api;

pub trait Vetter {
    async fn vet_order(rest_api_order: rest_api::trading::Order) -> VettingResult;
}

pub struct VettingResult {
    pub pass: bool
}