use crate::{auth::Claims, errors::ServiceError, settings::Settings};
use actix_web::{web, FromRequest};
use futures::future::{err, ok, ready, Ready};
#[derive(Debug, Clone, Serialize)]
pub struct ClaimsFromAuth(pub Claims);

fn verify_token(token: &str, settings: &Settings) -> Result<Claims, ServiceError> {
    let key = settings.auth.token_verification.create_decoding_key()?;

    let validation = jsonwebtoken::Validation::new(settings.auth.token_verification.as_algorithm());

    match jsonwebtoken::decode::<Claims>(token, &key, &validation) {
        Ok(token) => Ok(token.claims),
        Err(err) => {
            debug!("{}", err);
            Err(err.into())
        }
    }
}

impl FromRequest for ClaimsFromAuth {
    type Error = ServiceError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        if let Some(settings) = req.app_data::<web::Data<Settings>>()
            && let Some(authen_header) = req.headers().get("Authorization") {
                // Parse header
                if let Ok(authen_str) = authen_header.to_str()
                    && (authen_str.starts_with("bearer") || authen_str.starts_with("Bearer")) {
                        // Parse and verify token
                        let token = authen_str[6..authen_str.len()].trim();
                        return match verify_token(token, settings) {
                            // Use the verified claim
                            Ok(claim) => ok(ClaimsFromAuth(claim)),
                            // If a token was given but invalid, report an error
                            Err(e) => err(e),
                        };
                    }
            }

        // Return an anonymous default claim
        ready(Ok(ClaimsFromAuth(Claims {
            roles: vec![],
            groups: vec![],
            sub: "anonymous".to_string(),
            exp: None,
        })))
    }
}
