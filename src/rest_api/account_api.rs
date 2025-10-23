use crate::access_control::{AccessControl, Privilege};
use crate::constants::APPLICATION_JSON;
use crate::persistence::dao::Dao;
use crate::rest_api::base_api;
use actix_web::web::{Path, ThinData};
use actix_web::{HttpRequest, HttpResponse};

#[get("/accounts/{account_key}/positions")]
pub async fn get_positions(dao: ThinData<Dao>,
                           access_control: ThinData<AccessControl>,
                           account_key: Path<(String,)>,
                           req: HttpRequest,) -> HttpResponse {
    let account_key = &account_key.0.as_str().to_string();
    let customer_key = base_api::get_customer_key(req);

    if !access_control.is_allowed(account_key, customer_key, Privilege::Read).await {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;
    let positions = match txn.get_positions(account_key).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    let mut rest_api_positions = Vec::new();
    for position in positions {
        rest_api_positions.push(position.to_rest_api_position(account_key));
    }
    
    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(rest_api_positions)
}

#[get("/accounts/{account_key}/balances")]
pub async fn get_balance(dao: ThinData<Dao>,
                         access_control: ThinData<AccessControl>,
                         account_key: Path<(String,)>,
                         req: HttpRequest,) -> HttpResponse {
    let account_key = &account_key.0.as_str().to_string();
    let customer_key = base_api::get_customer_key(req);
    if !access_control.is_allowed(account_key, customer_key, Privilege::Read).await {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;
    let entities_balance = match txn.get_balances(account_key).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(Vec::new().push(entities_balance.to_rest_api_balance(account_key)))
}

#[get("/accounts/{account_key}")]
pub async fn get_account(dao: ThinData<Dao>,
                         access_control: ThinData<AccessControl>,
                         account_key: Path<(String,)>,
                         req: HttpRequest,) -> HttpResponse {
    let account_key = &account_key.0.as_str().to_string();
    let customer_key = base_api::get_customer_key(req);
    if !access_control.is_allowed(account_key, customer_key, Privilege::Read).await {
        return HttpResponse::Forbidden().finish();
    }
    let mut db_connection = dao.get_connection().await;
    let txn = dao.begin(&mut db_connection).await;
    let account = match txn.get_account_by_account_key(account_key).await {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(account.to_rest_api_account())
}
