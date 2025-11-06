use crate::access_control::AccessControl;
use crate::auth::templates::Welcome;
use crate::config::BrokerConfig;
use crate::persistence::dao::Dao;
use crate::rest_api::base_api::{log_dao_error_and_return_500, log_text_error_and_return_500};
use actix_session::Session;
use actix_web::http::header::{CONTENT_TYPE, LOCATION};
use actix_web::web::ThinData;
use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::Error;
use argonautica::{Hasher, Verifier};
use log::{debug, error, warn};
use serde::Deserialize;
use std::fmt::Display;

#[get("/")]
pub async fn welcome(
) -> HttpResponse {
    let template = Welcome {
        registration_failure_message: "",
        registration_success_message: "",
        login_failure_message: "",
    };

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(template.to_string())
}

fn registration_failure(message: &str) -> HttpResponse {
    let template = Welcome {
        registration_failure_message: message,
        registration_success_message: "",
        login_failure_message: "",
    };
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(template.to_string())
}

fn registration_success(message: &str) -> HttpResponse {
    let template = Welcome {
        registration_failure_message: "",
        registration_success_message: message,
        login_failure_message: "",
    };
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(template.to_string())
}

fn login_failure(message: &str) -> HttpResponse {
    let template = Welcome {
        registration_failure_message: "",
        registration_success_message: "",
        login_failure_message: message,
    };
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(template.to_string())
}

fn login_credential_failure() -> HttpResponse {
    login_failure("Your login credentials were invalid; please try again.")
}

fn login_technical_failure(failure: impl Display) -> HttpResponse {
    error!("Login failed: {}", failure);
    login_failure("Your login failed due to technical reasons; please try again.")
}

#[derive(Debug, Deserialize)]
pub struct RegisterData {
    pub offer_code: String,
    pub email_address: String,
    pub register_password: String,
    pub actor_name: String,
}

#[post("/register")]
pub async fn register(dao: ThinData<Dao>,
                      config: ThinData<BrokerConfig>,
                      req: HttpRequest,
                      data: web::Form<RegisterData>,
                      ) -> HttpResponse {
    debug!("Registering actor {} with offer code {}", data.email_address, data.offer_code);

    let mut db_connection = match dao.get_connection().await {
        Ok(db_connection) => db_connection,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(txn) => txn,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let offer_code_valid = match txn.check_offer(data.offer_code.as_str()).await {
        Ok(offer_code_valid) => offer_code_valid,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    if !offer_code_valid {
        return registration_failure("You have entered an invalid offer code; please try again.");
    }
    let password_hash = match hash_password(config.password_key.as_str(), data.register_password.as_str()) {
        Ok(password_hash) => password_hash,
        Err(hash_error) => return log_text_error_and_return_500(format!("Could not hash password: {}", hash_error)),
    };
    let actor = match txn.save_actor(data.email_address.as_str(), data.actor_name.as_str(), data.offer_code.as_str(), password_hash.as_str()).await {
        Ok(actor) => actor,
        Err(dao_error) => {
            warn!("User could not be registered: {}", dao_error);
            return registration_failure("You have entered invalid new credentials; please try again.");
        },
    };
    match txn.create_account_for_actor(&actor).await {
        Ok(_) => {},
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };

    match txn.commit().await {
        Ok(_) => {},
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };

    match is_json_request(&req) {
        true => HttpResponse::NoContent().finish(),
        false => registration_success("You have successfully registered and may now login."),
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

    let actor_option = match txn.get_actor_by_api_key(&data.api_key).await {
        Ok(actor_option) => actor_option,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let actor = match actor_option {
        Some(actor) => actor,
        None => return HttpResponse::Unauthorized().finish()
    };
    match access_control.set_current_actor(&txn, &session, &actor).await {
        Ok(_) => {}
        Err(set_error) => {
            session.clear();
            return log_text_error_and_return_500(format!("Failed to setup API actor {}: {}", &data.api_key, set_error));
        }
    }
    HttpResponse::Ok().json("{}")
}

#[derive(Debug, Deserialize)]
pub struct LoginData {
    pub email: String,
    pub login_password: String,
}

#[post("/login")]
pub async fn login(
    dao: ThinData<Dao>,
    session: Session,
    access_control: ThinData<AccessControl>,
    config: ThinData<BrokerConfig>,
    data: web::Form<LoginData>
) -> HttpResponse {
    debug!("Logging in actor {}", data.email);
    let mut db_connection = match dao.get_connection().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(x) => x,
        Err(dao_error) => return login_technical_failure(dao_error)
    };
    let actor_password_hash_option = match txn.get_actor_password_hash(data.email.as_str()).await {
        Ok(actor_password_hash_option) => actor_password_hash_option,
        Err(dao_error) => return login_technical_failure(dao_error)
    };
    let actor_password_hash = match actor_password_hash_option {
        Some(actor_password_hash) => actor_password_hash,
        None => return login_credential_failure()
    };

    let password_verified = match verify_password(config.password_key.as_str(), actor_password_hash.as_str(), data.login_password.as_str()) {
        Ok(password_verified) => password_verified,
        Err(verification_error) => return login_technical_failure(verification_error.to_string())

    };
    match password_verified {
        true => {}
        false => return login_credential_failure()
    };
    let actor_option = match txn.get_actor(data.email.as_str()).await {
        Ok(actor_option) => actor_option,
        Err(dao_error) => return login_technical_failure(dao_error)
    };
    let actor = match actor_option {
        Some(actor) => actor,
        None => return login_technical_failure("No actor")
    };

    match access_control.set_current_actor(&txn, &session, &actor).await {
        Ok(_) => {}
        Err(set_error) => {
            session.clear();
            return login_technical_failure(set_error)
        }
    }
    HttpResponse::SeeOther().append_header((LOCATION, "/app")).finish()
}

#[get("/logmeout")]
pub async fn logout(
    session: Session,
    req: HttpRequest) -> HttpResponse {
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

fn hash_password(key: &str, password: &str) -> Result<String, Error> {
    Hasher::default()
        .with_password(password)
        .with_secret_key(key)
        .hash()
        .map_err(|_| anyhow::anyhow!("Failed to hash password"))
}

pub fn verify_password(key: &str, hash: &str, password: &str) -> Result<bool, Error> {
    Verifier::default()
        .with_hash(hash)
        .with_password(password)
        .with_secret_key(key)
        .verify()
        .map_err(|_| anyhow::anyhow!("Could not verify password"))
}