use crate::{actions, errors::ServiceError, AppState};
use actix_web::web::{self, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueryParameters {
    q: String,
    query_language: String,
    corpora: String,
}

pub async fn count(
    info: web::Query<QueryParameters>,
    state: web::Data<AppState>,
    credentials: BearerAuth,
) -> Result<HttpResponse, ServiceError> {
    let corpora = vec![info.corpora.clone()];

    let conn = state
        .sqlite_pool
        .get()
        .expect("couldn't get db connection from pool");
    let requested_corpora = corpora.clone();
    let jwt_secret = state.settings.auth.jwt_secret.clone();
    let access_allowed: bool = web::block(move || {
        actions::corpus_access_allowed(requested_corpora, credentials.token(), &jwt_secret, &conn)
    })
    .await
    .map_err(|_| ServiceError::InternalServerError)?;

    if access_allowed {
        let count = state
            .cs
            .count(
                &corpora,
                &info.q,
                graphannis::corpusstorage::QueryLanguage::AQL,
            )
            .unwrap();
        Ok(HttpResponse::Ok().body(format!("{}", count)))
    } else {
        Err(ServiceError::NonAuthorizedCorpus)
    }
}
