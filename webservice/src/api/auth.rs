use crate::settings::Settings;
use actix_web::{error::ErrorInternalServerError, web, Error, HttpResponse};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::Deserialize;
use sha2::Sha256;

#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub user_id: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    /// Expiration date as unix timestamp in seconds since epoch and UTC
    pub exp: Option<i64>,
    pub corpus_groups: Vec<String>,
    pub admin: bool,
}

pub async fn local_login(
    login_data: web::Json<LoginData>,
    settings: web::Data<Settings>,
) -> Result<HttpResponse, Error> {
    // Check if user ID is set in configuration
    let provided_user = &login_data.user_id;
    if let Some(user) = settings.users.get(provided_user) {
        // Add Salt to password, calculate hash and compare against our settings
        let provided_password = &login_data.password;
        let verified =
            bcrypt::verify(&provided_password, &user.password).map_err(ErrorInternalServerError)?;
        if verified {
            // Create the JWT token
            let key: Hmac<Sha256> = Hmac::new_varkey(settings.auth.jwt_secret.as_bytes())
                .map_err(ErrorInternalServerError)?;
            // Determine an expiration date based on the configuration
            let now: chrono::DateTime<_> = chrono::Utc::now();
            let exp: Option<i64> = now
                .checked_add_signed(chrono::Duration::minutes(settings.auth.expiration_minutes))
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