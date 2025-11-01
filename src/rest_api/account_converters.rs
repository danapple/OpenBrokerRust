use crate::entities;
use crate::instrument_manager::InstrumentManager;
use crate::rest_api::account::{Account, Balance, Position};

impl entities::account::Account {
    pub fn to_rest_api_account(&self, nickname: &str) -> Account {
        Account {
            account_key: self.account_key.clone(),
            account_number: self.account_number.clone(),
            account_name: self.account_name.clone(),
            nickname: "".to_string(),
            privileges: Vec::new()
        }
    }
}

impl entities::account::Position {
    pub fn to_rest_api_position(&self, account_key: &str, instrument_manager: &InstrumentManager) -> Position {
        Position {
            account_key: account_key.to_string(),
            instrument_key: instrument_manager.get_instrument(self.instrument_id).unwrap().unwrap().instrument_key,
            quantity: self.quantity,
            cost: self.cost,
            closed_gain: self.closed_gain,
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

impl entities::account::Access {
    pub fn to_rest_api_account(&self, account: &entities::account::Account) -> Account {
        Account {
            account_key: account.account_key.to_string(),
            account_number: account.account_number.to_string(),
            account_name: account.account_name.to_string(),
            nickname: self.nickname.to_string(),
            privileges: Vec::new(),
        }
    }
}