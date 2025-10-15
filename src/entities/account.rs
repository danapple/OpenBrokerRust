pub struct Position {
    pub account_key: String,
    pub instrument_id: i64,
    pub quantity: i32,
    pub cost: f64,
}

pub struct Balance {
    pub account_key: String,
    pub cash: f64,
}
