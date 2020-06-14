use actix_web::{error::ResponseError, HttpResponse};
use graphannis::errors::GraphAnnisError;
use hmac::crypto_mac::InvalidKeyLength;
use std::fmt::Display;

#[derive(Debug)]
pub enum ServiceError {
    BadRequest(String),
    InvalidJWTToken(String),
    NonAuthorizedCorpus(Vec<String>),
    DatabaseError,
    InternalServerError,
    GraphAnnisError(GraphAnnisError),
    NotFound,
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::BadRequest(msg) => write!(f, "Bad Request: {}", msg)?,
            ServiceError::InvalidJWTToken(msg) => write!(f, "Invalid JWT Token: {}", msg)?,
            ServiceError::NonAuthorizedCorpus(corpora) => write!(
                f,
                "Not authorized to access the corpus/corpora {}",
                corpora.join(", ")
            )?,
            ServiceError::DatabaseError => write!(f, "Error accessing database")?,
            ServiceError::InternalServerError => write!(f, "Internal Server Error")?,
            ServiceError::GraphAnnisError(err) => write!(f, "{}", err)?,
            ServiceError::NotFound => write!(f, "Not found",)?,
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
            ServiceError::NonAuthorizedCorpus(corpora) => HttpResponse::Forbidden().json(format!(
                "Not authorized to access the corpus/corpora {}",
                corpora.join(", ")
            )),
            ServiceError::DatabaseError => {
                HttpResponse::BadGateway().json("Error accessing database")
            }
            ServiceError::InternalServerError => HttpResponse::InternalServerError().finish(),
            ServiceError::GraphAnnisError(err) => HttpResponse::BadRequest().json(err),
            ServiceError::NotFound => HttpResponse::NotFound().finish(),
        }
    }
}

impl From<InvalidKeyLength> for ServiceError {
    fn from(_: InvalidKeyLength) -> Self {
        ServiceError::BadRequest("Invalid JWT key length".to_owned())
    }
}

impl From<bcrypt::BcryptError> for ServiceError {
    fn from(_: bcrypt::BcryptError) -> Self {
        ServiceError::InternalServerError
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

impl From<std::io::Error> for ServiceError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => ServiceError::NotFound,
            _ => ServiceError::InternalServerError,
        }
    }
}
