use crate::persistence::dao::Dao;
use anyhow::Error;
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Deserialize, Serialize, Clone, ToSql, FromSql, PartialEq)]
pub enum Privilege {
    Owner,
    Read,
    Submit,
    Cancel,
}

impl Display for Privilege {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
pub struct AccessControl {
    dao: Dao,
}

impl AccessControl {
    pub fn new(dao: Dao) -> AccessControl {
        AccessControl {
            dao
        }
    }
    pub async fn is_allowed(& self, account_key: &str, api_key: Option<String>, privilege: Privilege) -> Result<bool, Error> {
        match api_key {
            Some(key) => {
                let mut db_connection = match self.dao.get_connection().await {
                    Ok(db_connection) => db_connection,
                    Err(dao_error) => return Err(anyhow::anyhow!("Could not get connection: {}", dao_error.to_string()))
                };
                let txn = match self.dao.begin(&mut db_connection).await {
                    Ok(txn) => txn,
                    Err(dao_error) => return Err(anyhow::anyhow!("Could not begin transaction: {}", dao_error.to_string()))
                };
                match txn.is_allowed(account_key, key.as_str(), privilege).await {
                    Ok(allowed) => Ok(allowed),
                    Err(dao_error) => Err(anyhow::anyhow!("Could not check is_allowed: {}", dao_error.to_string()))
                }
            }
            None => {
                Ok(false)
            }
        }
    }
}
