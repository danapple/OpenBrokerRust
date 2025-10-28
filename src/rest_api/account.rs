use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Privilege {
    Owner,
    Read,
    Submit,
    Cancel,
    Withdraw,
}

impl Display for Privilege {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Privilege {
    type Err = ();
    fn from_str(input: &str) -> Result<Privilege, Self::Err> {
        match input {
            "Owner"  => Ok(Privilege::Owner),
            "Read"  => Ok(Privilege::Read),
            "Submit"  => Ok(Privilege::Submit),
            "Cancel"  => Ok(Privilege::Cancel),
            "Withdraw"  => Ok(Privilege::Withdraw),
            _  => Err(()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub account_key: String,
    pub account_number: String,
    pub account_name: String,
    pub nickname: String,
    pub privileges: Vec<Privilege>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Position {
    pub account_key: String,
    pub instrument_id: i64,
    pub quantity: i32,
    pub cost: f32,
    pub version_number: i64,
    pub closed_gain: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Balance {
    pub account_key: String,
    pub cash: f32,
    pub version_number: i64,
}
