use actix_rt::blocking::BlockingError;
use actix_web::{error::ResponseError, HttpResponse};
use graphannis::errors::GraphAnnisError;
use std::fmt::Display;

#[derive(Debug)]
pub enum ServiceError {
    BadRequest(String),
    InvalidJWTToken(String),
    NoSuchCorpus(Vec<String>),
    DatabaseError(String),
    InternalServerError(String),
    GraphAnnisError(GraphAnnisError),
    NotFound,
    NotAnAdministrator(String),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::BadRequest(msg) => write!(f, "Bad Request: {}", msg)?,
            ServiceError::InvalidJWTToken(msg) => write!(f, "Invalid JWT Token: {}", msg)?,
            ServiceError::NoSuchCorpus(corpora) => {
                write!(f, "Non existing corpus/corpora {}", corpora.join(", "))?
            }
            ServiceError::DatabaseError(e) => write!(f, "Error accessing database: {}", e)?,
            ServiceError::InternalServerError(msg) => {
                write!(f, "Internal Server Error: {:?}", msg)?
            }
            ServiceError::GraphAnnisError(err) => write!(f, "{}", err)?,
            ServiceError::NotFound => write!(f, "Not found",)?,
            ServiceError::NotAnAdministrator(user) => {
                write!(f, "User {} is not an adminstrator", user)?
            }
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
        if let Some(graphannis_err) = orig.downcast_ref::<GraphAnnisError>() {
            match graphannis_err {
                GraphAnnisError::NoSuchCorpus(corpora) => {
                    ServiceError::NoSuchCorpus(vec![corpora.to_owned()])
                }
                _ => ServiceError::GraphAnnisError(graphannis_err.clone()),
            }
        } else {
            ServiceError::InternalServerError(orig.to_string())
        }
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
