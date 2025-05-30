use actix_web::{http::StatusCode, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError{
    #[error("Cant bind to the Socket")]
    SocketBind,
    #[error("Cant connect to the DB")]
    DbConnect,
    #[error("Cant start the server")]
    ServerStart
}


impl ResponseError for AppError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        actix_web::HttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            AppError::DbConnect => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ServerStart => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SocketBind => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}