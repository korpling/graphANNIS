use actix_web::{error::ResponseError, HttpResponse};
use hmac::crypto_mac::InvalidKeyLength;
use std::fmt::Display;

#[derive(Debug)]
pub enum ServiceError {
    BadRequest(String),
    InvalidJWTToken(String),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::BadRequest(msg) => write!(f, "Bad Request: {}", msg)?,
            ServiceError::InvalidJWTToken(msg) => write!(f, "Invalid JWT Token: {}", msg)?,
        }
        Ok(())
    }
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
            ServiceError::InvalidJWTToken(ref message) => {
                HttpResponse::Unauthorized().json(message)
            }
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
