pub struct Position {
    pub account_id: i64,
    pub instrument_id: i64,
    pub quantity: i32,
    pub cost: f64,
}

pub struct Account {
    pub account_id: i64,
    pub account_key: String,
    pub account_number: String,
    pub account_name: String,
}

pub struct Balance {
    pub account_id: i64,
    pub cash: f64,
}
