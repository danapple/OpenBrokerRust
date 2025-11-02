use crate::entities;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Offer {
    pub code: String,
    pub description: String,
    pub expiration_time: i64,
}

impl Offer {
    pub fn to_entities_offer(&self) -> entities::offer::Offer {
        entities::offer::Offer {
            offer_id: 0,
            code: self.code.clone(),
            description: self.description.clone(),
            expiration_time: self.expiration_time,
        }
    }
}