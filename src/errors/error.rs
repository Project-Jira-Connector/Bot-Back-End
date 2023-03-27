#[derive(Debug, serde::Serialize)]
pub struct Error {
    pub code: u16,
    pub description: String,
}

impl Error {
    pub fn new(code: actix_web::http::StatusCode, description: String) -> Self {
        return Self {
            code: code.as_u16(),
            description,
        };
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, format: &mut std::fmt::Formatter) -> std::fmt::Result {
        return write!(format, "{}", self);
    }
}

impl std::error::Error for Error {}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        return actix_web::http::StatusCode::from_u16(self.code)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        return actix_web::HttpResponse::build(self.status_code()).json(self);
    }
}

impl std::convert::From<Error> for actix_web::HttpResponse {
    fn from(error: Error) -> Self {
        return actix_web::ResponseError::error_response(&error);
    }
}
