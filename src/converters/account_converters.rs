use crate::dtos::account::{Account, Balance, Position};
use crate::entities;
use crate::instrument_manager::InstrumentManager;

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
        let instrument = match instrument_manager.get_instrument_by_exchange_instrument_id(self.instrument_id) {
            Ok(instrument) => instrument,
            Err(_) => todo!(),
        };
        let instrument_key = match instrument {
            Some(instrument_key) => instrument_key,
            None => todo!(),
        }.instrument_key;

        Position {
            account_key: account_key.to_string(),
            instrument_key,
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
