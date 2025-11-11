use crate::access_control::AccessControl;
use actix_session::Session;
use actix_web::http::header::{CONTENT_TYPE, LOCATION};
use actix_web::web::ThinData;
use actix_web::{HttpRequest, HttpResponse};

#[get("/logmeout")]
pub async fn logout(
    session: Session,
    access_control: ThinData<AccessControl>,
    req: HttpRequest) -> HttpResponse {
    access_control.clear(&session);
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
