use std::time::Duration;

use super::check_corpora_authorized;
use crate::{errors::ServiceError, extractors::ClaimsFromAuth, settings::Settings, DbPool};
use actix_web::web::{self, Bytes, HttpResponse};
use futures::stream::iter;
use graphannis::{
    corpusstorage::{FrequencyDefEntry, QueryLanguage, ResultOrder},
    CorpusStorage,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CountQuery {
    query: String,
    #[serde(default)]
    query_language: QueryLanguage,
    corpora: Vec<String>,
}

pub async fn count(
    params: web::Json<CountQuery>,
    cs: web::Data<CorpusStorage>,
    db_pool: web::Data<DbPool>,
    settings: web::Data<Settings>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    let corpora = check_corpora_authorized(params.corpora.clone(), claims.0, &db_pool).await?;
    let count = cs.count_extra(
        &corpora,
        &params.query,
        params.query_language,
        settings
            .database
            .query_timeout
            .map(|secs| Duration::from_secs(secs)),
    )?;
    Ok(HttpResponse::Ok().json(count))
}

#[derive(Deserialize)]
pub struct ParseQuery {
    query: String,
    #[serde(default)]
    query_language: QueryLanguage,
}

pub async fn node_descriptions(
    params: web::Query<ParseQuery>,
    cs: web::Data<CorpusStorage>,
) -> Result<HttpResponse, ServiceError> {
    let desc = cs.node_descriptions(&params.query, params.query_language)?;
    Ok(HttpResponse::Ok().json(desc))
}

#[derive(Deserialize)]
pub struct FindQuery {
    query: String,
    #[serde(default)]
    query_language: QueryLanguage,
    corpora: Vec<String>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    offset: usize,
    #[serde(default)]
    order: ResultOrder,
}

pub async fn find(
    params: web::Json<FindQuery>,
    cs: web::Data<CorpusStorage>,
    db_pool: web::Data<DbPool>,
    settings: web::Data<Settings>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    let corpora = check_corpora_authorized(params.corpora.clone(), claims.0, &db_pool).await?;

    let matches = cs.find(
        &corpora,
        &params.query,
        params.query_language,
        params.offset,
        params.limit,
        params.order,
        settings
            .database
            .query_timeout
            .map(|secs| Duration::from_secs(secs)),
    )?;

    let body = iter(
        matches
            .into_iter()
            .map(|mut line| -> Result<_, ServiceError> {
                line.push('\n');
                Ok(Bytes::from(line))
            }),
    );
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .streaming(body))
}

#[derive(Deserialize)]
pub struct FrequencyQuery {
    query: String,
    #[serde(default)]
    query_language: QueryLanguage,
    corpora: Vec<String>,
    definition: Vec<FrequencyDefEntry>,
}

pub async fn frequency(
    params: web::Json<FrequencyQuery>,
    cs: web::Data<CorpusStorage>,
    db_pool: web::Data<DbPool>,
    settings: web::Data<Settings>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    let corpora = check_corpora_authorized(params.corpora.clone(), claims.0, &db_pool).await?;

    let result = cs.frequency(
        &corpora,
        &params.query,
        params.query_language,
        params.definition.clone(),
        settings
            .database
            .query_timeout
            .map(|secs| Duration::from_secs(secs)),
    )?;

    Ok(HttpResponse::Ok().json(result))
}
