use crate::dtos::account::{Account, Privilege};
use crate::dtos::actor::Power;
use crate::entities::actor::Actor;
use crate::persistence::dao::DaoTransaction;
use actix_session::Session;
use anyhow::Error;
use log::{debug, info};
use std::collections::HashMap;

const SESSION_ACTOR_KEY: &'static str = "actor";
const SESSION_ACCOUNT_MAP_KEY: &'static str = "accounts";
const SESSION_POWERS: &'static str = "powers";

#[derive(Clone)]
pub struct AccessControl {
}

impl AccessControl {
    pub fn new() -> AccessControl {
        AccessControl {
        }
    }

    pub(crate) fn clear(&self,
                        session: &Session) {
        session.remove(SESSION_ACTOR_KEY);
        session.remove(SESSION_ACCOUNT_MAP_KEY);
        session.remove(SESSION_POWERS);
        session.clear();
    }

    pub(crate) async fn set_current_actor(&self, 
                                          txn: &DaoTransaction<'_>, 
                                          session: &Session, 
                                          actor: &Actor) -> Result<(), Error> {
        debug!("set_current_actor using session {:p}", session);

        let account_map = match self.build_account_map(txn, actor).await {
            Ok(account_map) => account_map,
            Err(build_error) => return Err(build_error),
        };
        let powers = match self.build_powers(txn, actor).await {
            Ok(power_map) => power_map,
            Err(build_error) => return Err(build_error),
        };
        // info!("Got powers: {:?}", powers);
        match session.insert(SESSION_ACTOR_KEY, actor) {
            Ok(_) => { },
            Err(insert_error) => return Err(anyhow::anyhow!("set_current_actor failed to insert actor into session: {}", insert_error)),
        };

        match session.insert(SESSION_ACCOUNT_MAP_KEY, account_map) {
            Ok(_) => { },
            Err(insert_error) => {
                session.clear();
                return Err(anyhow::anyhow!("set_current_actor failed to insert account map into session: {}", insert_error))
            },
        };
        match session.insert(SESSION_POWERS, powers) {
            Ok(_) => { },
            Err(insert_error) => {
                session.clear();
                return Err(anyhow::anyhow!("set_current_actor failed to insert power into session: {}", insert_error))
            },
        };
        Ok(())
    }

    pub fn get_allowed_accounts(&self, 
                                session: &Session) -> Result<HashMap<String, Account>, Error> {
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

    pub fn is_allowed_from_map(&self, 
                               allowed_accounts: &HashMap<String, Account>, 
                               account_key: &str, 
                               privilege: Privilege) -> Result<bool, Error> {
        let account = match allowed_accounts.get(account_key) {
            Some(account) => account,
            None => return Ok(false)
        };
        Ok(account.privileges.contains(&privilege))
    }

    pub fn is_allowed(&self, session: &Session) -> Result<bool, Error> {
        let actor = match session.get::<Actor>(SESSION_ACTOR_KEY) {
            Ok(actor) => actor,
            Err(get_error) => return Err(anyhow::anyhow!("Could not get actor: {}", get_error.to_string()))
        };
        match actor {
            Some(_) => Ok(true),
            None => Ok(false)
        }
    }

    pub fn is_allowed_account_privilege(&self, 
                                        session: &Session, 
                                        account_key: &str, 
                                        privilege: Privilege) -> Result<bool, Error> {
        debug!("is_allowed checking account_key {} with privilege {} against session", account_key, privilege);
        let accounts = match self.get_allowed_accounts(session) {
            Ok(accounts) => accounts,
            Err(get_allowed_error) => return Err(anyhow::anyhow!("Could not get_allowed_accounts: {}", get_allowed_error.to_string()))
        };
        self.is_allowed_from_map(&accounts, account_key, privilege)
    }

    pub fn is_admin_allowed_power(& self, 
                                  session: &Session, 
                                  power: Power) -> Result<bool, Error> {
        debug!("is_admin_allowed checking with power {} against session", power);
        let powers_option  = match session.get::<Vec<Power>>(SESSION_POWERS) {
            Ok(powers) => powers,
            Err(get_error) => return Err(anyhow::anyhow!("Could not get power: {}", get_error.to_string()))
        };
        let powers = match powers_option {
            Some(powers) => powers,
            None => return Err(anyhow::anyhow!("No powers available"))
        };
        Ok(powers.contains(&power))
    }

    async fn build_account_map(&self, 
                               txn: &DaoTransaction<'_>, 
                               actor: &Actor) -> Result<HashMap<String, Account>, Error> {
        let accesses = match txn.get_accesses_for_actor(actor.actor_id).await {
            Ok(accesses) => accesses,
            Err(dao_error) => return Err(anyhow::anyhow!("build_account_map failed to get accesses for actor: {}", dao_error)),
        };

        let account_ids: Vec<i32> = accesses.iter().map(|access| access.account_id).collect();

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

    async fn build_powers(&self, txn: &DaoTransaction<'_>, actor: &Actor) -> Result<Vec<Power>, Error> {
        let powers = match txn.get_powers(actor.actor_id).await {
            Ok(powers) => powers,
            Err(dao_error) => return Err(anyhow::anyhow!("build_powers failed to get powers for actor: {}", dao_error)),
        };
        Ok(powers)
    }
}
