use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Customer {
        pub customer_id: i64,
        pub email: String,
    }
