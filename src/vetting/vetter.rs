use crate::dtos;

pub trait Vetter {
    async fn vet_order(rest_api_order: dtos::order::Order) -> VettingResult;
}

pub struct VettingResult {
    pub pass: bool
}