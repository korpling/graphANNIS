use actix_web::{error::ResponseError, HttpResponse};
use graphannis::errors::GraphAnnisError;
use hmac::crypto_mac::InvalidKeyLength;
use std::fmt::Display;

#[derive(Debug)]
pub enum ServiceError {
    BadRequest(String),
    InvalidJWTToken(String),
    NonAuthorizedCorpus,
    DatabaseError,
    InternalServerError,
    GraphAnnisError(GraphAnnisError),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::BadRequest(msg) => write!(f, "Bad Request: {}", msg)?,
            ServiceError::InvalidJWTToken(msg) => write!(f, "Invalid JWT Token: {}", msg)?,
            ServiceError::NonAuthorizedCorpus => write!(f, "Not authorized to access corpus")?,
            ServiceError::DatabaseError => write!(f, "Error accessing database")?,
            ServiceError::InternalServerError => write!(f, "Internal Server Error")?,
            ServiceError::GraphAnnisError(err) => write!(f, "{}", err)?,
        }
        Ok(())
    }
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
            ServiceError::NonAuthorizedCorpus => {
                HttpResponse::Unauthorized().json("Not authorized to access the given corpus")
            }
            ServiceError::DatabaseError => {
                HttpResponse::BadGateway().json("Error accessing database")
            }
            ServiceError::InternalServerError => HttpResponse::InternalServerError().finish(),
            ServiceError::GraphAnnisError(err) => HttpResponse::BadRequest().json(err),
        }
    }
}

impl From<InvalidKeyLength> for ServiceError {
    fn from(_: InvalidKeyLength) -> Self {
        ServiceError::BadRequest("Invalid JWT key length".to_owned())
    }
}

impl From<jwt::Error> for ServiceError {
    fn from(orig: jwt::Error) -> Self {
        ServiceError::InvalidJWTToken(format!("{}", orig))
    }
}

impl From<diesel::result::Error> for ServiceError {
    fn from(_orig: diesel::result::Error) -> Self {
        ServiceError::DatabaseError
    }
}

impl From<anyhow::Error> for ServiceError {
    fn from(orig: anyhow::Error) -> Self {
        if let Some(graphannis_err) = orig.downcast_ref::<GraphAnnisError>() {
            ServiceError::GraphAnnisError(graphannis_err.clone())
        } else {
            ServiceError::InternalServerError
        }
    }
}
