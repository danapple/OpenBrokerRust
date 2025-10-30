use crate::access_control::AccessControl;
use crate::constants::APPLICATION_JSON;
use crate::rest_api::account::Account;
use actix_session::Session;
use actix_web::web::ThinData;
use actix_web::HttpResponse;
use log::error;

#[get("/accounts")]
pub async fn get_accounts(access_control: ThinData<AccessControl>,
                          session: Session) -> HttpResponse {
    let allowed_accounts_map = match access_control.get_allowed_accounts(&session) {
        Ok(allowed_accounts_map) => allowed_accounts_map,
        Err(error) => {
            error!("Failed while get accounts from session: {}", error.to_string());
            return HttpResponse::InternalServerError().finish();
        }
    };
    let account_vec: Vec<Account> = allowed_accounts_map.values().cloned().collect();

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(account_vec)
}