use crate::entities;
use crate::rest_api::account::{Account, Balance, Position};

impl entities::account::Account {
    pub fn to_rest_api_account(&self) -> Account {
        Account {
            account_key: self.account_key.clone(),
            account_number: self.account_number.clone(),
            account_name: self.account_name.clone(),
        }
    }
}

impl entities::account::Position {
    pub fn to_rest_api_position(&self, account_key: &str) -> Position {
        Position {
            account_key: account_key.to_string(),
            instrument_id: self.instrument_id,
            quantity: self.quantity,
            cost: self.cost,
            version_number: self.version_number,
        }
    }
}

impl entities::account::Balance {
    pub fn to_rest_api_balance(&self, account_key: &str) -> Balance {
        Balance {
            account_key: account_key.to_string(),
            cash: self.cash,
            version_number: self.version_number,
        }
    }
}