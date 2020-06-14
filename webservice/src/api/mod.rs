use crate::{actions, errors::ServiceError, DbPool};
use actix_web::web;
use auth::Claims;
use graphannis::corpusstorage::QueryLanguage;

pub mod auth;
pub mod corpora;
pub mod search;

fn check_is_admin(claims: &Claims) -> bool {
    claims.admin
}

/// Check that all `requested_corpora` are authorized for the user. If any of them is not, a `ServiceError::NonAuthorizedCorpus` error is returned.
async fn check_corpora_authorized(
    requested_corpora: Vec<String>,
    claims: Claims,
    db_pool: &web::Data<DbPool>,
) -> Result<Vec<String>, ServiceError> {
    if check_is_admin(&claims) {
        // Adminstrators always have access to all corpora
        return Ok(requested_corpora);
    }

    let conn = db_pool.get().map_err(|_| ServiceError::DatabaseError)?;
    let allowed_corpora =
        web::block(move || actions::authorized_corpora_from_groups(&claims, &conn))
            .await
            .map_err(|_| ServiceError::InternalServerError)?;

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

fn parse_query_language(query_language: &Option<String>) -> QueryLanguage {
    if let Some(query_language) = query_language {
        if query_language.to_uppercase() == "AQL_QUIRKS_V3" {
            return QueryLanguage::AQLQuirksV3;
        }
    }
    QueryLanguage::AQL
}

fn parse_corpora(corpora: &str) -> Vec<String> {
    corpora.split(",").map(|c| c.trim().to_string()).collect()
}
