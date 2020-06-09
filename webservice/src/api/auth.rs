use crate::AppState;
use actix_web::{error::ErrorInternalServerError, post, web, Error, HttpResponse};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::Deserialize;
use sha2::Sha256;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub username_or_email: String,
    pub password: String,
}

#[post("/login")]
async fn login(
    login_data: web::Json<LoginData>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    // Check if user ID is set in configuration
    let provided_user = &login_data.username_or_email;
    if let Some(user) = state.settings.users.get(provided_user) {
        // Add Salt to password, calculate hash and compare against our settings
        let provided_password = &login_data.password;
        let verified =
            bcrypt::verify(&provided_password, &user.password).map_err(ErrorInternalServerError)?;
        if verified {
            // Create the JWT token
            let key: Hmac<Sha256> = Hmac::new_varkey(state.settings.auth.jwt_secret.as_bytes())
                .map_err(ErrorInternalServerError)?;
            let mut claims: BTreeMap<_, &str> = BTreeMap::new();
            claims.insert("sub", provided_user);
            // Add the corpus groups and adminstrator status as claims
            let corpus_groups = &user.corpus_groups.join(",");
            claims.insert("corpus_groups", &corpus_groups);
            if user.admin {
                claims.insert("admin", "true");
            }
            // Create the actual token
            let token_str = claims
                .sign_with_key(&key)
                .map_err(ErrorInternalServerError)?;
            return Ok(HttpResponse::Ok().body(token_str));
        }
    }
    Ok(HttpResponse::Unauthorized().finish())
}
