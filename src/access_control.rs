use crate::persistence::dao::Dao;
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
    pub async fn is_allowed(& self, account_key: &str, customer_key: Option<String>, privilege: Privilege) -> bool {
        match customer_key {
            Some(key) => {
                let mut db_connection = self.dao.get_connection().await;
                let txn = self.dao.begin(&mut db_connection).await;
                match txn.is_allowed(account_key, key.as_str(), privilege).await {
                    Ok(x) => x,
                    Err(_) => todo!(),
                }
            }
            None => {
                false
            }
        }
    }
}
