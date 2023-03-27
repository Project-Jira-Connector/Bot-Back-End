use crate::*;

pub async fn not_found(_request: actix_web::HttpRequest) -> actix_web::HttpResponse {
    return errors::error::Error::new(
        actix_web::http::StatusCode::NOT_FOUND,
        "The requested resource could not be found on the server".to_string(),
    )
    .into();
}

pub async fn method_not_allowed(request: actix_web::HttpRequest) -> actix_web::HttpResponse {
    return errors::error::Error::new(
        actix_web::http::StatusCode::METHOD_NOT_ALLOWED,
        format!(
            "The requested resource does not support the HTTP method ({}) used in this request",
            request.method()
        ),
    )
    .into();
}

pub fn json(
    error: actix_web::error::JsonPayloadError,
    _: &actix_web::HttpRequest,
) -> actix_web::Error {
    return errors::error::Error::new(
        actix_web::http::StatusCode::BAD_REQUEST,
        format!("An error occurred while trying to parse the JSON data. Please ensure that your data is properly formatted and try again. ({})", error),
    )
    .into();
}

pub fn query(
    error: actix_web::error::QueryPayloadError,
    _: &actix_web::HttpRequest,
) -> actix_web::Error {
    return errors::error::Error::new(
        actix_web::http::StatusCode::BAD_REQUEST,
        format!("An error occurred while trying to parse the QUERY data. Please ensure that your data is properly formatted and try again. ({})", error),
    )
    .into();
}
