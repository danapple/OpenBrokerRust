use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Actor {
    pub actor_id: i32,
    pub email_address: String,
    pub actor_name: String,
    pub offer_code: Option<String>
}
