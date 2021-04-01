use crate::{actions, auth::Claims, errors::ServiceError, DbPool};
use actix_web::web;

pub mod administration;
pub mod corpora;
pub mod search;

fn check_is_admin(claims: &Claims) -> Result<(), ServiceError> {
    if claims.roles.iter().any(|r| r.as_str() == "admin") {
        Ok(())
    } else {
        Err(ServiceError::NotAnAdministrator(claims.sub.clone()))
    }
}

/// Check that all `requested_corpora` are authorized for the user. If any of them is not, a `ServiceError::NonAuthorizedCorpus` error is returned.
async fn check_corpora_authorized(
    requested_corpora: Vec<String>,
    claims: Claims,
    db_pool: &web::Data<DbPool>,
) -> Result<Vec<String>, ServiceError> {
    if claims.roles.iter().any(|r| r.as_str() == "admin") {
        // Administrators always have access to all corpora
        return Ok(requested_corpora);
    }

    let conn = db_pool.get()?;
    let allowed_corpora =
        web::block(move || actions::authorized_corpora_from_groups(&claims, &conn)).await?;

    if requested_corpora
        .iter()
        .all(|c| allowed_corpora.contains(c))
    {
        Ok(requested_corpora)
    } else {
        Err(ServiceError::NonAuthorizedCorpus(
            requested_corpora
                .into_iter()
                .filter(|c| !allowed_corpora.contains(c))
                .collect(),
        ))
    }
}
