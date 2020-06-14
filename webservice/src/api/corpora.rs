use super::check_is_admin;
use crate::{actions, errors::ServiceError, extractors::ClaimsFromAuth, DbPool};
use actix_web::web::{self, HttpResponse};
use graphannis::CorpusStorage;

pub async fn list(
    cs: web::Data<CorpusStorage>,
    claims: ClaimsFromAuth,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, ServiceError> {
    let all_corpora: Vec<String> = cs.list()?.into_iter().map(|c| c.name).collect();

    let allowed_corpora = if check_is_admin(&claims.0) {
        // Adminstrators always have access to all corpora
        all_corpora
    } else {
        // Query the database for all allowed corpora of this user
        let conn = db_pool.get().map_err(|_| ServiceError::DatabaseError)?;
        let corpora_by_group =
            web::block(move || actions::authorized_corpora_from_groups(&claims.0, &conn))
                .await
                .map_err(|_| ServiceError::InternalServerError)?;
        // Filter out non-existing corpora
        all_corpora
            .into_iter()
            .filter(|c| corpora_by_group.contains(c))
            .collect()
    };

    Ok(HttpResponse::Ok().json(allowed_corpora))
}
