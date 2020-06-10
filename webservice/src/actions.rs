use crate::{api::auth::Claims, errors::ServiceError};
use diesel::prelude::*;
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha256;

pub fn corpus_access_allowed(
    requested_corpora: &Vec<String>,
    token: &str,
    jwt_secret: &str,
    conn: &SqliteConnection,
) -> Result<bool, ServiceError> {
    use crate::schema::corpus_groups::dsl::*;

    let key: Hmac<Sha256> = Hmac::new_varkey(jwt_secret.as_bytes())?;

    let claims: Claims = VerifyWithKey::verify_with_key(token, &key)?;

    let allowed_corpora: Vec<String> = corpus_groups
        .filter(group.eq_any(&claims.corpus_groups))
        .load::<crate::models::CorpusGroup>(conn)?
        .iter()
        .map(|cg| cg.corpus.clone())
        .collect();

    // Check if all requested corpora are allowed
    let mut allowed = true;
    for c in requested_corpora {
        if !allowed_corpora.contains(&c) {
            allowed = false;
            break;
        }
    }

    Ok(allowed)
}
