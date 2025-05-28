use std::sync::PoisonError;

use actix_web::{
    error::{BlockingError, ResponseError},
    HttpResponse,
};
use graphannis::errors::{AQLError, GraphAnnisError};
use graphannis_core::errors::GraphAnnisCoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Invalid JWT Token: {0}")]
    InvalidJWTToken(String),
    #[error("Not authorized to access corpus/corpora {0:?}")]
    NonAuthorizedCorpus(Vec<String>),
    #[error("Error accessing database: {0}")]
    DatabaseError(String),
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    #[error(transparent)]
    GraphAnnisError(#[from] GraphAnnisError),
    #[error("Not found")]
    NotFound,
    #[error("User {0} is not an administrator")]
    NotAnAdministrator(String),
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
    #[error("{0}")]
    IllegalNodePath(String),
    #[error("Lock poisoning ({0})")]
    LockPoisoning(String),
}

impl<T> From<PoisonError<T>> for ServiceError {
    fn from(e: PoisonError<T>) -> Self {
        Self::LockPoisoning(e.to_string())
    }
}

#[derive(Serialize)]
enum BadRequestError {
    AQLSyntaxError(AQLError),
    AQLSemanticError(AQLError),
    ImpossibleSearch(String),
    Uuid(String),
    IllegalNodePath(String),
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InvalidJWTToken(ref message) => {
                HttpResponse::Unauthorized().json(message)
            }
            ServiceError::NonAuthorizedCorpus(corpora) => HttpResponse::Forbidden().json(format!(
                "Not authorized to access corpus/corpora {}",
                corpora.join(", ")
            )),
            ServiceError::DatabaseError(_) => {
                HttpResponse::BadGateway().json("Error accessing database")
            }
            ServiceError::InternalServerError(_) => HttpResponse::InternalServerError().finish(),
            ServiceError::Uuid(err) => {
                HttpResponse::BadRequest().json(BadRequestError::Uuid(err.to_string()))
            }
            ServiceError::IllegalNodePath(err) => {
                HttpResponse::BadRequest().json(BadRequestError::IllegalNodePath(err.to_string()))
            }
            ServiceError::GraphAnnisError(err) => match err {
                GraphAnnisError::Timeout => HttpResponse::GatewayTimeout().finish(),
                GraphAnnisError::AQLSemanticError(aql_error) => HttpResponse::BadRequest()
                    .json(BadRequestError::AQLSemanticError(aql_error.clone())),
                GraphAnnisError::AQLSyntaxError(aql_error) => HttpResponse::BadRequest()
                    .json(BadRequestError::AQLSyntaxError(aql_error.clone())),
                GraphAnnisError::ImpossibleSearch(aql_error) => HttpResponse::BadRequest()
                    .json(BadRequestError::ImpossibleSearch(aql_error.clone())),
                _ => HttpResponse::InternalServerError().json(err.to_string()),
            },
            ServiceError::NotFound => HttpResponse::NotFound().finish(),
            ServiceError::NotAnAdministrator(_) => HttpResponse::Forbidden()
                .json("You need to have administrator privilege to access this resource."),
            ServiceError::LockPoisoning(err) => {
                HttpResponse::InternalServerError().json(err.to_string())
            }
        }
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

impl From<BlockingError> for ServiceError {
    fn from(e: BlockingError) -> Self {
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

impl From<GraphAnnisCoreError> for ServiceError {
    fn from(e: GraphAnnisCoreError) -> Self {
        ServiceError::DatabaseError(e.to_string())
    }
}
