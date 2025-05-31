use actix_web::{http::StatusCode, ResponseError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use derive_more::derive::{Display, Error as DeriveMoreError};

#[derive(Debug, Error)]
pub enum AppError{
    #[error("Cant bind to the Socket")]
    SocketBind,
    #[error("Cant connect to the DB")]
    DbConnect,
    #[error("Cant start the server")]
    ServerStart,
    #[error("Internal Server Error")]
    InternalError
}

#[derive(Debug, Display, DeriveMoreError, Serialize, Deserialize)]
#[display("error :{}", error)]
pub struct CustomError{
    pub error:String
}

impl ResponseError for CustomError{}

impl ResponseError for AppError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        actix_web::HttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            AppError::DbConnect => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ServerStart => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SocketBind => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}