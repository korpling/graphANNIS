use crate::AppState;
use actix_web::{
    error::ErrorInternalServerError, http, http::header, post, web, Error, HttpResponse,
};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::Deserialize;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub username_or_email: String,
    pub password: String,
    pub redirect_to: Option<String>,
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
            if let Some(redirect_to) = &login_data.redirect_to {
                let token_parameter_name = &state.settings.auth.redirect_token_parameter;
                let redirect_to = redirect_to
                    .parse::<http::Uri>()
                    .map_err(ErrorInternalServerError)?;
                // Regenerate the redirect URI, but rewrite the query component to include our token
                let mut redirect_to = redirect_to.into_parts();
                let original_path = redirect_to
                    .path_and_query
                    .map(|p| p.path().to_owned())
                    .unwrap_or_default();
                let path_and_query =
                    format!("{}?{}={}", original_path, token_parameter_name, token_str);
                redirect_to.path_and_query = Some(
                    http::uri::PathAndQuery::try_from(path_and_query.as_str())
                        .map_err(ErrorInternalServerError)?,
                );
                return Ok(HttpResponse::TemporaryRedirect()
                    .header(
                        header::LOCATION,
                        http::Uri::from_parts(redirect_to)
                            .map_err(ErrorInternalServerError)?
                            .to_string(),
                    )
                    .finish());
            } else {
                return Ok(HttpResponse::Ok().body(token_str));
            }
        }
    }
    Ok(HttpResponse::Unauthorized().finish())
}
