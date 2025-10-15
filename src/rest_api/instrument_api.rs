use actix_web::{web, HttpResponse};
use actix_web::web::ThinData;
use deadpool_postgres::Pool;
use log::info;
use crate::constants::APPLICATION_JSON;

#[get("/instrument")]
pub async fn get_instruments(ThinData(db_pool): web::ThinData<Pool>) -> HttpResponse {
    info!("get_instruments called");
    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json("[]")
}
