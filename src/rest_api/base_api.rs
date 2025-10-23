use actix_web::HttpRequest;

pub fn get_customer_key(req: HttpRequest,) -> Option<String> {
    match req.cookie("customer_key") {
        Some(cookie) => {
            Some(cookie.value().to_string())
        }
        None => {
            None
        }
    }
}