use crate::dtos::account::{Account, Balance, Position};
use crate::entities;
use crate::instrument_manager::InstrumentManager;
use anyhow::Error;

impl entities::account::Account {
    pub fn to_rest_api_account(&self, 
                               nickname: &str) -> Account {
        Account {
            account_key: self.account_key.clone(),
            account_number: self.account_number.clone(),
            account_name: self.account_name.clone(),
            nickname: nickname.to_string(),
            privileges: Vec::new()
        }
    }
}

impl entities::account::Position {
    pub fn to_rest_api_position(&self, 
                                account_key: &str, 
                                instrument_manager: &InstrumentManager) -> Result<Position, Error> {
        let instrument_option = instrument_manager.get_instrument(self.instrument_id)?;
        let instrument = match instrument_option {
            Some(instrument_key) => instrument_key,
            None => return Err(anyhow::anyhow!("No instrument for instrument id {}", self.instrument_id))
        };

        Ok(Position {
            account_key: account_key.to_string(),
            instrument_key: instrument.instrument_key,
            quantity: self.quantity,
            cost: self.cost,
            closed_gain: self.closed_gain,
            version_number: self.version_number,
        })
    }
}

impl entities::account::Balance {
    pub fn to_rest_api_balance(&self, 
                               account_key: &str) -> Balance {
        Balance {
            account_key: account_key.to_string(),
            cash: self.cash,
            version_number: self.version_number,
        }
    }
}
