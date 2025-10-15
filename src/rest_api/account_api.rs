
use actix_web::HttpResponse;
use actix_web::web::{Json, Path, ThinData};
use crate::access_control::{AccessControl, Privilege};
use crate::constants::{APPLICATION_JSON};
use crate::persistence::dao::Dao;
use crate::rest_api;
use crate::rest_api::base_api;

#[get("/accounts/{account_key}/positions")]
pub async fn get_positions(dao: ThinData<Dao>,
                           access_control: ThinData<AccessControl>,
                           account_key: Path<(String,)>) -> HttpResponse {
    let account_key = &account_key.0.as_str().to_string();
    let customer_key = base_api::get_customer_key();

    if !access_control.is_allowed(account_key, &customer_key, Privilege::Read) {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;
    let positions = match txn.get_positions(account_key).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    
    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json("")
}

#[get("/accounts/{account_key}/balances")]
pub async fn get_balance(dao: ThinData<Dao>,
                         access_control: ThinData<AccessControl>,
                         account_key: Path<(String,)>) -> HttpResponse {
    let account_key = &account_key.0.as_str().to_string();
    let customer_key = base_api::get_customer_key();

    if !access_control.is_allowed(account_key, &customer_key, Privilege::Read) {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;
    let entities_balance = match txn.get_balances(account_key).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    let balance = rest_api::account::Balance {
        cash: entities_balance.cash,
    };
    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(Vec::new().push(balance))
}

