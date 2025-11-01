use crate::access_control::AccessControl;
use crate::entities::exchange::instrument_from_exchange_instrument;
use crate::exchange_interface::exchange_client::ExchangeClient;
use crate::instrument_manager::InstrumentManager;
use crate::persistence::dao::Dao;
use crate::rest_api;
use crate::rest_api::actor::Power;
use crate::rest_api::base_api::{log_dao_error_and_return_500, log_text_error_and_return_500};
use actix_session::Session;
use actix_web::web::{Json, Path, ThinData};
use actix_web::HttpResponse;
use log::{error, info};
use std::sync::Arc;

#[post("/admin/exchange")]
pub async fn create_exchange(dao: ThinData<Dao>,
                             mut instrument_manager: ThinData<InstrumentManager>,
                             access_control: ThinData<AccessControl>,
                             session: Session,
                             exchange: Json<rest_api::exchange::Exchange>,
) -> HttpResponse {
    info!("create_exchange called");

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
    let mut db_exchange = exchange.to_entities_exchange();
    match txn.save_exchange(&mut db_exchange).await {
        Ok(_) => {}
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    match instrument_manager.setup_exchange(db_exchange).await {
        Ok(_) => {},
        Err(setup_error) => return log_text_error_and_return_500(setup_error.to_string()),
    };
    match txn.commit().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    HttpResponse::Ok().finish()
}

#[put("/admin/exchange/{exchange_code}")]
pub async fn load_exchange_instruments(dao: ThinData<Dao>,
                                       mut instrument_manager: ThinData<InstrumentManager>,
                                       access_control: ThinData<AccessControl>,
                                       session: Session,
                                       path: Path<(String)>,
) -> HttpResponse {
    info!("load_exchange_instruments called");

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

    let exchange_code = path.into_inner();

    let exchange = match txn.get_exchange(exchange_code.as_str()).await {
        Ok(exchange) => exchange,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };

    let exchange_client = Arc::new(ExchangeClient::new(exchange.url.as_str(), exchange.api_key.as_str()));
    let instruments = match exchange_client.clone().get_instruments().await {
        Ok(instruments) => instruments,
        Err(instrument_error) => {
            error!("Should error getting instruments from the exchange: {}", instrument_error);
            return HttpResponse::InternalServerError().finish();
        },
    };

    for instrument in instruments.instruments.values() {
        info!("Adding instrument: {} for exchange {}", instrument.instrument_id, exchange.code);
        let mut db_instrument = instrument_from_exchange_instrument(instrument, exchange.exchange_id);
        match txn.save_instrument(&mut db_instrument).await {
            Ok(_) => {},
            Err(dao_error) => return log_dao_error_and_return_500(dao_error),
        };
        match instrument_manager.add_instrument(&db_instrument) {
            Ok(x) => x,
            Err(err) => return log_text_error_and_return_500(err.to_string()),
        };
    }

    match txn.commit().await {
        Ok(x) => x,
        Err(dao_error) => return log_dao_error_and_return_500(dao_error),
    };
    HttpResponse::Ok().finish()
}

