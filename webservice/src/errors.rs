use actix_rt::blocking::BlockingError;
use actix_web::{error::ResponseError, HttpResponse};
use graphannis::errors::GraphAnnisError;
use graphannis_core::errors::GraphAnnisCoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Invalid JWT Token: {0}")]
    InvalidJWTToken(String),
    #[error("Non existing corpus/corpora {0:?}")]
    NoSuchCorpus(Vec<String>),
    #[error("Error accessing database: {0}")]
    DatabaseError(String),
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    #[error("{0}")]
    GraphAnnisError(String),
    #[error("Not found")]
    NotFound,
    #[error("User {0} is not an adminstrator")]
    NotAnAdministrator(String),
    #[error("Timeout")]
    Timeout,
}

#[derive(Serialize)]
pub struct AQLErrorBody {}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
            ServiceError::InvalidJWTToken(ref message) => {
                HttpResponse::Unauthorized().json(message)
            }
            ServiceError::NoSuchCorpus(corpora) => HttpResponse::NotFound().json(format!(
                "Non existing corpus/corpora {}",
                corpora.join(", ")
            )),
            ServiceError::DatabaseError(_) => {
                HttpResponse::BadGateway().json("Error accessing database")
            }
            ServiceError::InternalServerError(_) => HttpResponse::InternalServerError().finish(),
            ServiceError::GraphAnnisError(err) => HttpResponse::BadRequest().json(err),
            ServiceError::NotFound => HttpResponse::NotFound().finish(),
            ServiceError::NotAnAdministrator(_) => HttpResponse::Forbidden()
                .json("You need to have administrator privilege to access this resource."),
            ServiceError::Timeout => HttpResponse::GatewayTimeout().finish(),
        }
    }
}

impl From<bcrypt::BcryptError> for ServiceError {
    fn from(e: bcrypt::BcryptError) -> Self {
        ServiceError::InternalServerError(format!("{}", e))
    }
}

impl From<jsonwebtoken::errors::Error> for ServiceError {
    fn from(orig: jsonwebtoken::errors::Error) -> Self {
        ServiceError::InvalidJWTToken(format!("{}", orig))
    }
}

impl From<diesel::result::Error> for ServiceError {
    fn from(orig: diesel::result::Error) -> Self {
        ServiceError::DatabaseError(orig.to_string())
    }
}

impl From<r2d2::Error> for ServiceError {
    fn from(orig: r2d2::Error) -> Self {
        ServiceError::DatabaseError(orig.to_string())
    }
}

impl From<anyhow::Error> for ServiceError {
    fn from(orig: anyhow::Error) -> Self {
        ServiceError::InternalServerError(orig.to_string())
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => ServiceError::NotFound,
            _ => ServiceError::InternalServerError(e.to_string()),
        }
    }
}

impl From<std::path::StripPrefixError> for ServiceError {
    fn from(e: std::path::StripPrefixError) -> Self {
        ServiceError::InternalServerError(e.to_string())
    }
}

impl From<walkdir::Error> for ServiceError {
    fn from(e: walkdir::Error) -> Self {
        ServiceError::InternalServerError(e.to_string())
    }
}

impl<T: std::fmt::Debug> From<BlockingError<T>> for ServiceError {
    fn from(e: BlockingError<T>) -> Self {
        ServiceError::InternalServerError(e.to_string())
    }
}

impl From<actix_web::Error> for ServiceError {
    fn from(e: actix_web::Error) -> Self {
        ServiceError::InternalServerError(e.to_string())
    }
}

impl From<actix_web::error::PayloadError> for ServiceError {
    fn from(e: actix_web::error::PayloadError) -> Self {
        ServiceError::InternalServerError(e.to_string())
    }
}

impl From<zip::result::ZipError> for ServiceError {
    fn from(e: zip::result::ZipError) -> Self {
        ServiceError::InternalServerError(e.to_string())
    }
}

impl From<uuid::Error> for ServiceError {
    fn from(e: uuid::Error) -> Self {
        ServiceError::BadRequest(e.to_string())
    }
}

impl From<GraphAnnisCoreError> for ServiceError {
    fn from(e: GraphAnnisCoreError) -> Self {
        ServiceError::DatabaseError(e.to_string())
    }
}

impl From<GraphAnnisError> for ServiceError {
    fn from(e: GraphAnnisError) -> Self {
        match e {
            GraphAnnisError::NoSuchCorpus(c) => ServiceError::NoSuchCorpus(vec![c]),
            GraphAnnisError::Timeout => ServiceError::Timeout,
            _ => ServiceError::GraphAnnisError(e.to_string()),
        }
    }
}
