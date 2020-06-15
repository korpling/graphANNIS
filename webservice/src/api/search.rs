use super::{check_corpora_authorized};
use crate::{errors::ServiceError, extractors::ClaimsFromAuth, DbPool};
use actix_web::web::{self, HttpResponse};
use graphannis::{
    corpusstorage::{QueryLanguage, ResultOrder},
    CorpusStorage,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FindQuery {
    query: String,
    #[serde(default)]
    query_language: QueryLanguage,
    corpora: Vec<String>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    order: ResultOrder,
}

pub async fn find(
    params: web::Query<FindQuery>,
    cs: web::Data<CorpusStorage>,
    db_pool: web::Data<DbPool>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    let corpora = check_corpora_authorized(params.corpora.clone(), claims.0, &db_pool).await?;


    let matches = cs.find(
        &corpora,
        &params.query,
        params.query_language,
        params.offset.unwrap_or_default(),
        params.limit,
        params.order,
    )?;
    Ok(HttpResponse::Ok().body(matches.join("\n")))
}

#[derive(Deserialize)]
pub struct Query {
    query: String,
    #[serde(default)]
    query_language: QueryLanguage,
    corpora: Vec<String>,
}

pub async fn count(
    params: web::Json<Query>,
    cs: web::Data<CorpusStorage>,
    db_pool: web::Data<DbPool>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    let corpora = check_corpora_authorized(params.corpora.clone(), claims.0, &db_pool).await?;
    let count = cs.count_extra(&corpora, &params.query, params.query_language)?;
    Ok(HttpResponse::Ok().json(count))
}
