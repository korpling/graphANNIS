use crate::{errors::ServiceError, AppState};
use actix_web::{error::ErrorInternalServerError, web, Error, HttpResponse};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use serde::Deserialize;
use sha2::Sha256;

#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub user_id: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    /// Expiration date as unix timestamp in seconds since epoch and UTC
    exp: Option<i64>,
    corpus_groups: Vec<String>,
    admin: bool,
}

pub fn corpus_access_allowed(corpora : &[&str], token: &str, state: web::Data<AppState>) -> Result<bool, ServiceError> {
    let key: Hmac<Sha256> = Hmac::new_varkey(state.settings.auth.jwt_secret.as_bytes())?;

    let claims = VerifyWithKey::verify_with_key(token, &key)?;
    Ok(claims)
}

pub fn validate_token(token: &str, state: web::Data<AppState>) -> Result<bool, ServiceError> {
    let key: Hmac<Sha256> = Hmac::new_varkey(state.settings.auth.jwt_secret.as_bytes())?;

    let claims: Claims = VerifyWithKey::verify_with_key(token, &key)?;
    if let Some(exp) = claims.exp {
        // Check that the claim is still valid, thus not expired
        let expiration_date = chrono::NaiveDateTime::from_timestamp(exp, 0);
        let current_date = chrono::Utc::now();
        Ok(current_date.naive_utc() < expiration_date)
    } else {
        Ok(true)
    }
}

pub async fn local_login(
    login_data: web::Json<LoginData>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    // Check if user ID is set in configuration
    let provided_user = &login_data.user_id;
    if let Some(user) = state.settings.users.get(provided_user) {
        // Add Salt to password, calculate hash and compare against our settings
        let provided_password = &login_data.password;
        let verified =
            bcrypt::verify(&provided_password, &user.password).map_err(ErrorInternalServerError)?;
        if verified {
            // Create the JWT token
            let key: Hmac<Sha256> = Hmac::new_varkey(state.settings.auth.jwt_secret.as_bytes())
                .map_err(ErrorInternalServerError)?;
            // Determine an expiration date based on the configuration
            let now: chrono::DateTime<_> = chrono::Utc::now();
            let exp: Option<i64> = now
                .checked_add_signed(chrono::Duration::minutes(
                    state.settings.auth.expiration_minutes,
                ))
                .map(|d| d.timestamp());

            let claims = Claims {
                sub: provided_user.clone(),
                corpus_groups: user.corpus_groups.clone(),
                admin: user.admin,
                exp,
            };
            // Create the actual token
            let token_str = claims
                .sign_with_key(&key)
                .map_err(ErrorInternalServerError)?;
            return Ok(HttpResponse::Ok().body(token_str));
        }
    }
    Ok(HttpResponse::Unauthorized().finish())
}
