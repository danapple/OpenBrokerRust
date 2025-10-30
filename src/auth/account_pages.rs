use actix_session::Session;
use actix_web::http::header::{CONTENT_TYPE, LOCATION};
use actix_web::web::ThinData;
use actix_web::{web, HttpRequest, HttpResponse};
use argonautica::{Hasher, Verifier};
use log::debug;
use serde::Deserialize;

use crate::access_control::AccessControl;
use crate::auth::templates::Welcome;
use crate::config::BrokerConfig;
use crate::persistence::dao::Dao;
use crate::rest_api::base_api::{log_dao_error_and_return_500, log_text_error_and_return_500};

#[get("/")]
pub async fn welcome(
) -> HttpResponse {
    let template = Welcome {  };

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(template.to_string())
}


#[derive(Debug, Deserialize)]
pub struct RegisterData {
    pub offer_code: String,
    pub email_address: String,
    pub password: String,
    pub customer_name: String,
}

#[post("/register")]
pub async fn register(
                      dao: ThinData<Dao>,
                      config: ThinData<BrokerConfig>,
                      req: HttpRequest,
                      data: web::Form<RegisterData>,
                      ) -> HttpResponse {
    debug!("Registering user {} with offer code {}", data.email_address, data.offer_code);

    let mut db_connection = match dao.get_connection().await {
        Ok(db_connection) => db_connection,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(txn) => txn,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let offer_code_valid = match txn.check_offer_code(data.offer_code.as_str()).await {
        Ok(offer_code_valid) => offer_code_valid,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    if !offer_code_valid {
        return HttpResponse::TemporaryRedirect().append_header((LOCATION, "/")).finish()
    }
    let password_hash = match hash_password(config.password_key.as_str(), data.password.as_str()) {
        Ok(password_hash) => password_hash,
        Err(hash_error) => return log_text_error_and_return_500(format!("Could not hash password: {}", hash_error)),
    };
    match txn.save_customer(data.email_address.as_str(), data.customer_name.as_str(), data.offer_code.as_str(), password_hash.as_str()).await {
        Ok(_) => { },
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    match txn.commit().await {
        Ok(_) => {},
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    match is_json_request(&req) {
        true => HttpResponse::NoContent().finish(),
        false => HttpResponse::TemporaryRedirect().append_header((LOCATION, "/")).finish(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiLoginData {
    pub api_key: String,
}

#[post("/loginapi")]
pub async fn loginapi(
    dao: ThinData<Dao>,
    session: Session,
    access_control: ThinData<AccessControl>,
    data: web::Json<ApiLoginData>,
) -> HttpResponse {
    debug!("Logging in API {}", &data.api_key);
    let mut db_connection = match dao.get_connection().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };

    let customer_option = match txn.get_customer_by_api_key(&data.api_key).await {
        Ok(customer_option) => customer_option,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let customer = match customer_option {
        Some(customer) => customer,
        None => return HttpResponse::Unauthorized().finish()
    };
    match access_control.set_current_user(&txn, &session, &customer).await {
        Ok(_) => {}
        Err(set_error) => {
            session.clear();
            return log_text_error_and_return_500(format!("Failed to setup API user {}: {}", &data.api_key, set_error));
        }
    }
    HttpResponse::Ok().json("{}")
}

#[derive(Debug, Deserialize)]
pub struct LoginData {
    pub email: String,
    pub password: String,
}

#[post("/login")]
pub async fn login(
    dao: ThinData<Dao>,
    session: Session,
    access_control: ThinData<AccessControl>,
    config: ThinData<BrokerConfig>,
    data: web::Form<LoginData>
) -> HttpResponse {
    debug!("Logging in user {}", data.email);
    let mut db_connection = match dao.get_connection().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let customer_password_hash_option = match txn.get_customer_password_hash(data.email.as_str()).await {
        Ok(customer_password_hash_option) => customer_password_hash_option,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let customer_password_hash = match customer_password_hash_option {
        Some(customer_password_hash) => customer_password_hash,
        None =>
            return log_text_error_and_return_500(format!("Password could not be retrieved for {}", data.email)),
    };

    let password_verified = match verify_password(config.password_key.as_str(), customer_password_hash.as_str(), data.password.as_str()) {
        Ok(password_verified) => password_verified,
        Err(verification_error) =>
            return log_text_error_and_return_500(format!("Error verifying password for {}: {}", data.email, verification_error)),
    };
    match password_verified {
        true => {}
        false => return HttpResponse::TemporaryRedirect().append_header((LOCATION, "/")).finish()
    };
    let customer_option = match txn.get_customer(data.email.as_str()).await {
        Ok(customer_option) => customer_option,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let customer = match customer_option {
        Some(customer) => customer,
        None => return HttpResponse::TemporaryRedirect().append_header((LOCATION, "/")).finish()
    };

    match access_control.set_current_user(&txn, &session, &customer).await {
        Ok(_) => {}
        Err(set_error) => {
            session.clear();
            return log_text_error_and_return_500(format!("Failed to setup user {}: {}", data.email, set_error));
        }
    }
    HttpResponse::SeeOther().append_header((LOCATION, "/app")).finish()
}

#[get("/logmeout")]
pub async fn logout(
    session: Session,
    req: HttpRequest,
) -> HttpResponse {
    session.clear();
    // TODO terminate websockets
    match is_json_request(&req) {
        true => HttpResponse::NoContent().finish(),
        false => HttpResponse::TemporaryRedirect().append_header((LOCATION, "/")).finish(),
    }
}

fn is_json_request(req: &HttpRequest) -> bool {
    req
        .headers()
        .get(CONTENT_TYPE)
        .map_or(
            false,
            |header| header.to_str().map_or(false, |content_type| "application/json" == content_type)
        )
}

fn hash_password(key: &str, password: &str) -> Result<String, anyhow::Error> {
    Hasher::default()
        .with_password(password)
        .with_secret_key(key)
        .hash()
        .map_err(|_| anyhow::anyhow!("Failed to hash password"))
}

pub fn verify_password(key: &str, hash: &str, password: &str) -> Result<bool, anyhow::Error> {
    Verifier::default()
        .with_hash(hash)
        .with_password(password)
        .with_secret_key(key)
        .verify()
        .map_err(|_| anyhow::anyhow!("Could not verify password"))
}