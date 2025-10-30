use crate::constants::{SESSION_ACCOUNT_MAP_KEY, SESSION_USER_KEY};
use crate::entities::customer::Customer;
use crate::persistence::dao::DaoTransaction;
use crate::rest_api::account::{Account, Privilege};
use actix_session::Session;
use anyhow::Error;
use log::debug;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AccessControl {
}

impl AccessControl {
    pub fn new() -> AccessControl {
        AccessControl {
        }
    }

    pub(crate) async fn set_current_user(&self, txn: &DaoTransaction<'_>, session: &Session, customer: &Customer) -> Result<(), Error> {
        debug!("set_current_user using session {:p}", session);

        let account_map = match self.build_account_map(txn, customer).await {
            Ok(account_map) => account_map,
            Err(build_error) => return Err(build_error),
        };
        match session.insert(SESSION_USER_KEY, customer) {
            Ok(_) => { },
            Err(insert_error) => return Err(anyhow::anyhow!("set_current_user failed to insert user into session: {}", insert_error)),
        };

        match session.insert(SESSION_ACCOUNT_MAP_KEY, account_map) {
            Ok(_) => { },
            Err(insert_error) => {
                session.clear();
                return Err(anyhow::anyhow!("set_current_user failed to insert accesses into session: {}", insert_error))
            },
        };
        Ok(())
    }

    pub fn get_allowed_accounts(&self, session: &Session) -> Result<HashMap<String, Account>, Error> {
        debug!("get_allowed_accounts using session {:p}", session);


        let account_map_option  = match session.get::<HashMap<String, Account>>(SESSION_ACCOUNT_MAP_KEY) {
            Ok(account_map_option) => account_map_option,
            Err(get_error) => return Err(anyhow::anyhow!("Could not get account map: {}", get_error.to_string()))
        };
        let account_map = match account_map_option {
            Some(account_map) => account_map,
            None => return Err(anyhow::anyhow!("No account map available"))
        };
        Ok(account_map)
    }

    pub fn is_allowed_from_map(&self, allowed_accounts: &HashMap<String, Account>, account_key: &str, privilege: Privilege) -> Result<bool, Error> {
        let account = match allowed_accounts.get(account_key) {
            Some(account) => account,
            None => return Ok(false)
        };
        Ok(account.privileges.contains(&privilege))
    }

    pub async fn is_allowed(& self, session: &Session, account_key: &str, privilege: Privilege) -> Result<bool, Error> {
        debug!("is_allowed checking account_key {} with privilege {} against session", account_key, privilege);
        let accounts = match self.get_allowed_accounts(session) {
            Ok(accounts) => accounts,
            Err(get_allowed_error) => return Err(anyhow::anyhow!("Could not get_allowed_accounts: {}", get_allowed_error.to_string()))
        };
        self.is_allowed_from_map(&accounts, account_key, privilege)
    }

    async fn build_account_map(&self, txn: &DaoTransaction<'_>, customer: &Customer) -> Result<HashMap<String, Account>, Error> {
        let accesses = match txn.get_accesses_for_customer(customer.customer_id).await {
            Ok(accesses) => accesses,
            Err(dao_error) => return Err(anyhow::anyhow!("build_account_map failed to get accesses for customer: {}", dao_error)),
        };

        let account_ids: Vec<i64> = accesses.iter().map(|access| access.account_id).collect();

        let accounts = match txn.get_accounts(account_ids).await {
            Ok(accounts) => accounts,
            Err(dao_error) => return Err(anyhow::anyhow!("build_account_map failed to get accounts: {}", dao_error)),
        };

        let mut account_map: HashMap<String, Account> = HashMap::new();

        for access_db in accesses {
            let account = match accounts.get(&access_db.account_id) {
                Some(account) => account,
                None => return Err(anyhow::anyhow!("Account {} not found in accounts", access_db.account_id)),
            };
            if !account_map.contains_key(&account.account_key) {
                let new_rest_api_account = account.to_rest_api_account(access_db.nickname.as_str());
                account_map.insert(account.account_key.clone(), new_rest_api_account);
            }
            let rest_api_account = match account_map.get_mut(&account.account_key) {
                Some(rest_api_account) => rest_api_account,
                None => return Err(anyhow::anyhow!("Account {} not found in access_map", access_db.account_id)),
            };
            rest_api_account.privileges.push(access_db.privilege);
        }
        Ok(account_map)
    }
}
