use std::time::Duration;

use super::check_corpora_authorized_read;
use crate::{errors::ServiceError, extractors::ClaimsFromAuth, settings::Settings, DbPool};
use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};
use futures::stream::iter;
use graphannis::{
    corpusstorage::{FrequencyDefEntry, QueryLanguage, ResultOrder, SearchQuery},
    CorpusStorage,
};
use serde::Deserialize;

#[derive(Deserialize, Serialize, Debug)]
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
    let corpora =
        check_corpora_authorized_read(params.corpora.clone(), claims.0, &settings, &db_pool)
            .await?;
    let query = SearchQuery {
        corpus_names: &corpora,
        query: &params.query,
        query_language: params.query_language,
        timeout: settings.database.query_timeout.map(Duration::from_secs),
    };

    let count = cs.count_extra(query)?;
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
    let corpora =
        check_corpora_authorized_read(params.corpora.clone(), claims.0, &settings, &db_pool)
            .await?;
    let query = SearchQuery {
        corpus_names: &corpora,
        query: &params.query,
        query_language: params.query_language,
        timeout: settings.database.query_timeout.map(Duration::from_secs),
    };
    let matches = cs.find(query, params.offset, params.limit, params.order)?;

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
    let corpora =
        check_corpora_authorized_read(params.corpora.clone(), claims.0, &settings, &db_pool)
            .await?;
    let query = SearchQuery {
        corpus_names: &corpora,
        query: &params.query,
        query_language: params.query_language,
        timeout: settings.database.query_timeout.map(Duration::from_secs),
    };
    let result = cs.frequency(query, params.definition.clone())?;

    Ok(HttpResponse::Ok().json(result))
}

#[cfg(test)]
mod tests;
