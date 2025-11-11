use crate::access_control::AccessControl;
use crate::auth::account_pages::{verify_password, LoginData, RegisterData};
use crate::config::BrokerConfig;
use crate::persistence::dao::Dao;
use crate::rest_api::base_api::{log_anyhow_error_and_return_500, log_dao_error_and_return_500, log_text_error_and_return_500};
use actix_session::Session;
use actix_web::web::ThinData;
use actix_web::{web, HttpResponse};
use log::{debug, info, warn};

#[post("/login_ui")]
pub async fn login_ui(
    dao: ThinData<Dao>,
    session: Session,
    access_control: ThinData<AccessControl>,
    config: ThinData<BrokerConfig>,
    data: web::Json<LoginData>
) -> HttpResponse {
    info!("Logging in actor {}", data.email_address);
    let mut db_connection = match dao.get_connection().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    let txn = match dao.begin(&mut db_connection).await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error)
    };
    let actor_password_hash_option = match txn.get_actor_password_hash(data.email_address.as_str()).await {
        Ok(actor_password_hash_option) => actor_password_hash_option,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error)
    };
    let actor_password_hash = match actor_password_hash_option {
        Some(actor_password_hash) => actor_password_hash,
        None => return HttpResponse::Unauthorized().finish()
    };

    let password_verified = match verify_password(config.password_key.as_str(), actor_password_hash.as_str(), data.password.as_str()) {
        Ok(password_verified) => password_verified,
        Err(verification_error) => return log_anyhow_error_and_return_500(verification_error)

    };
    match password_verified {
        true => {}
        false => return HttpResponse::Unauthorized().finish()
    };
    let actor_option = match txn.get_actor(data.email_address.as_str()).await {
        Ok(actor_option) => actor_option,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error)
    };
    let actor = match actor_option {
        Some(actor) => actor,
        None => return log_text_error_and_return_500("no actor, but password was retrieved")
    };

    match access_control.set_current_actor(&txn, &session, &actor).await {
        Ok(_) => {}
        Err(set_error) => {
            session.clear();
            return log_anyhow_error_and_return_500(set_error)
        }
    }
    HttpResponse::Ok().finish()
}

#[post("/register_ui")]
pub async fn register_ui(dao: ThinData<Dao>,
                         config: ThinData<BrokerConfig>,
                         data: web::Json<RegisterData>,
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
        return HttpResponse::NotFound().finish();
    }
    let password_hash = match crate::auth::account_pages::hash_password(config.password_key.as_str(), data.password.as_str()) {
        Ok(password_hash) => password_hash,
        Err(hash_error) => return log_text_error_and_return_500(format!("Could not hash password: {}", hash_error).as_str()),
    };
    let actor = match txn.save_actor(data.email_address.as_str(), data.actor_name.as_str(), data.offer_code.as_str(), password_hash.as_str()).await {
        Ok(actor) => actor,
        Err(dao_error) => {
            warn!("User could not be registered: {}", dao_error);
            return HttpResponse::NotAcceptable().finish();
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

    HttpResponse::Created().finish()
}
