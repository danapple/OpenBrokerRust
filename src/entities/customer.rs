use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Customer {
    pub customer_id: i64,
    pub email_address: String,
    pub customer_name: String,
    pub offer_code: Option<String>
}
