use crate::access_control::AccessControl;
use actix_session::Session;
use actix_web::http::header::{CONTENT_TYPE, LOCATION};
use actix_web::web::ThinData;
use actix_web::{HttpRequest, HttpResponse};

#[post("/logout")]
pub async fn logout(
    session: Session,
    access_control: ThinData<AccessControl>) -> HttpResponse {
    access_control.clear(&session);
    // TODO terminate websockets
    HttpResponse::Ok().finish()
}
