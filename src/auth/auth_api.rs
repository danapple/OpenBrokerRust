use crate::access_control::AccessControl;
use crate::persistence::dao::Dao;
use crate::rest_api::base_api::{log_dao_error_and_return_500, log_text_error_and_return_500};
use actix_session::Session;
use actix_web::web::ThinData;
use actix_web::{web, HttpResponse};
use log::debug;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct ApiLoginData {
    pub api_key: String,
}

#[post("/login_api")]
pub async fn login_api(
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
            return log_text_error_and_return_500(format!("Failed to setup API actor {}: {}", &data.api_key, set_error).as_str());
        }
    }
    HttpResponse::Ok().json("{}")
}
