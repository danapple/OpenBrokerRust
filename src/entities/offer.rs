use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Offer {
    pub offer_id: i32,
    pub code: String,
    pub description: String,
    pub expiration_time: i64,
}
