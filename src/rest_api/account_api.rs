use crate::constants::APPLICATION_JSON;
use crate::persistence::dao::Dao;
use crate::rest_api;
use crate::rest_api::base_api;
use crate::rest_api::base_api::{log_dao_error_and_return_500, log_text_error_and_return_500};
use actix_web::web::ThinData;
use actix_web::{HttpRequest, HttpResponse};
use log::error;
use std::collections::HashMap;

#[get("/accounts")]
pub async fn get_accounts(dao: ThinData<Dao>,
                          req: HttpRequest,) -> HttpResponse {
    let api_key = match base_api::get_api_key(req) {
        Some(api_key) => api_key,
        None => return HttpResponse::PreconditionFailed().finish()
    };

    let mut db_connection = match dao.get_connection().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let accesses = match txn.get_accesses(api_key.as_str()).await {
        Ok(accesses) => accesses,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };

    let account_ids: Vec<i64> = accesses.iter().map(|access| access.account_id).collect();
    
    let accounts = match txn.get_accounts(account_ids).await {
        Ok(accounts) => accounts,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };

    let mut account_map: HashMap<i64, rest_api::account::Account> = HashMap::new();

    for access_db in accesses {
        if !account_map.contains_key(&access_db.account_id) {
            let account = match accounts.get(&access_db.account_id) {
                Some(account) => account,
                None => return log_text_error_and_return_500(format!("Account {} not found", access_db.account_id)),
            };
            let new_rest_api_account = account.to_rest_api_account(access_db.nickname.as_str());
            account_map.insert(access_db.account_id, new_rest_api_account);
        }
        let rest_api_account = match account_map.get_mut(&access_db.account_id) {
            Some(rest_api_account) => rest_api_account,
            None => return log_text_error_and_return_500(format!("Account {} not found in access_map", access_db.account_id)),
        };
        rest_api_account.privileges.push(access_db.privilege);
    }

    match txn.rollback().await {
        Ok(_) => {},
        Err(error) => {
            error!("Failed while rolling back: {}", error.to_string());
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut accounts = Vec::new();
    for account in account_map.values() {
        accounts.push(account);
    }
    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(accounts)
}