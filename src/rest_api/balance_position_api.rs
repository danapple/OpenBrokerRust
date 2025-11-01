use crate::access_control::AccessControl;
use crate::constants::APPLICATION_JSON;
use crate::instrument_manager::InstrumentManager;
use crate::persistence::dao::Dao;
use crate::rest_api::account::Privilege;
use crate::rest_api::base_api;
use crate::rest_api::base_api::log_dao_error_and_return_500;
use actix_session::Session;
use actix_web::web::{Path, ThinData};
use actix_web::{HttpRequest, HttpResponse};
use log::error;
use std::collections::HashMap;

#[get("/accounts/{account_key}/positions")]
pub async fn get_positions(dao: ThinData<Dao>,
                           instrument_manager: ThinData<InstrumentManager>,
                           access_control: ThinData<AccessControl>,
                           session: Session,
                           account_key: Path<(String,)>,) -> HttpResponse {
    let account_key = &account_key.0.as_str().to_string();

    let allowed: bool = match access_control.is_allowed_account_privilege(&session, &account_key, Privilege::Read) {
        Ok(allowed) => allowed,
        Err(error) => {
            error!("Failed while checking access: {}", error.to_string());
            return HttpResponse::InternalServerError().finish();
        }
    };
    if !allowed {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = match dao.get_connection().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let positions = match txn.get_positions(account_key).await {
        Ok(x) => x,
        Err(y) => {
            error!("get_positions error: {}", y);
            return HttpResponse::NotFound()
                .content_type(APPLICATION_JSON)
                .finish();
        },
    };
    let mut rest_api_positions = HashMap::new();
    for position in positions.values() {
        rest_api_positions.insert(position.position_id, position.to_rest_api_position(account_key, &instrument_manager));
    }

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(rest_api_positions)
}

#[get("/accounts/{account_key}/balances")]
pub async fn get_balance(dao: ThinData<Dao>,
                         access_control: ThinData<AccessControl>,
                         session: Session,
                         account_key: Path<(String,)>,) -> HttpResponse {
    let account_key = &account_key.0.as_str().to_string();
    let allowed: bool = match access_control.is_allowed_account_privilege(&session, &account_key, Privilege::Read) {
        Ok(allowed) => allowed,
        Err(error) => {
            error!("Failed while checking access: {}", error.to_string());
            return HttpResponse::InternalServerError().finish();
        }
    };
    if !allowed {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = match dao.get_connection().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    match txn.get_balance(account_key).await {
        Ok(balance) => {
            HttpResponse::Ok()
                .content_type(APPLICATION_JSON)
                .json(Vec::new().push(balance.to_rest_api_balance(account_key)))
        }
        Err(y) => {
            error!("get_balance error {}", y);
            HttpResponse::NotFound()
                .content_type(APPLICATION_JSON)
                .finish()
        }
    }
}