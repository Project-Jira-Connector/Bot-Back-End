use crate::*;

pub async fn handler() -> actix_web::HttpResponse {
    return errors::error::Error::new(
        actix_web::http::StatusCode::NOT_FOUND,
        "The page you are looking for doesn't exist".to_string(),
    )
    .into();
}

pub fn json_handler(
    error: actix_web::error::JsonPayloadError,
    _: &actix_web::HttpRequest,
) -> actix_web::Error {
    return errors::error::Error::new(
        actix_web::http::StatusCode::BAD_REQUEST,
        format!("Failed to parse your json input ({})", error),
    )
    .into();
}
