use crate::access_control::AccessControl;
use crate::persistence::dao::Dao;
use crate::rest_api::actor::Power;
use crate::rest_api::base_api::log_dao_error_and_return_500;
use crate::rest_api::offer::Offer;
use actix_session::Session;
use actix_web::web::{Json, ThinData};
use actix_web::HttpResponse;
use log::{error, info};

#[post("/admin/offer")]
pub async fn create_offer(dao: ThinData<Dao>,
                          access_control: ThinData<AccessControl>,
                          session: Session,
                          offer_code: Json<Offer>,
) -> HttpResponse {
    info!("create_offer_code called");

    let allowed: bool = match access_control.is_admin_allowed(&session, Power::All).await {
        Ok(allowed) => allowed,
        Err(error) => {
            error!("Failed while checking admin access: {}", error.to_string());
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
    match txn.save_offer(offer_code.to_entities_offer()).await {
        Ok(_) => {}
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    match txn.commit().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    HttpResponse::Ok().finish()
}

