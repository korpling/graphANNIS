use crate::{errors::ServiceError, settings::Settings};
use actix_web::{web, HttpResponse};
use serde::Deserialize;

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
    pub groups: Vec<String>,
    pub roles: Vec<String>,
}

pub async fn local_login(
    login_data: web::Json<LoginData>,
    settings: web::Data<Settings>,
) -> Result<HttpResponse, ServiceError> {
    // Check if user ID is set in configuration
    let provided_user = &login_data.user_id;
    if let Some(user) = settings.users.get(provided_user) {
        // Add Salt to password, calculate hash and compare against our settings
        let provided_password = &login_data.password;
        let verified = bcrypt::verify(&provided_password, &user.password)?;
        if verified {
            // Determine an expiration date based on the configuration
            let now: chrono::DateTime<_> = chrono::Utc::now();
            let exp: i64 = now
                .checked_add_signed(chrono::Duration::minutes(settings.auth.expiration_minutes))
                .ok_or_else(|| {
                    ServiceError::InternalServerError(
                        "Could not add expiration time to current time".to_string(),
                    )
                })?
                .timestamp();

            let roles = if user.admin {
                vec!["admin".to_string()]
            } else {
                vec![]
            };

            let claims = Claims {
                sub: provided_user.clone(),
                groups: user.corpus_groups.clone(),
                roles,
                exp: Some(exp),
            };
            // Create the JWT key and header from the configuration
            let key = settings.auth.token_verification.create_encoding_key()?;
            let header = jsonwebtoken::Header::new(settings.auth.token_verification.as_algorithm());
            // Create the actual token
            let token_str = jsonwebtoken::encode(&header, &claims, &key)?;
            return Ok(HttpResponse::Ok()
                .content_type("text/plain")
                .body(token_str));
        }
    }
    Ok(HttpResponse::Unauthorized().finish())
}
