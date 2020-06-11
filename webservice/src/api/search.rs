use crate::{actions, errors::ServiceError, extractors::ClaimsAuth, DbPool};
use actix_web::web::{self, HttpResponse};
use graphannis::{
    corpusstorage::{QueryLanguage, ResultOrder},
    CorpusStorage,
};
use serde::Deserialize;

async fn corpus_access_allowed(
    requested_corpora: Vec<String>,
    claims: ClaimsAuth,
    sqlite_pool: web::Data<DbPool>,
) -> Result<bool, ServiceError> {
    let conn = sqlite_pool.get().map_err(|_| ServiceError::DatabaseError)?;
    let access_allowed: bool =
        web::block(move || actions::corpus_access_allowed(&requested_corpora, &claims.0, &conn))
            .await
            .map_err(|_| ServiceError::InternalServerError)?;
    Ok(access_allowed)
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

#[derive(Deserialize)]
pub struct FindQueryParameters {
    q: String,
    #[serde(default)]
    query_language: Option<String>,
    corpora: String,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    order: Option<String>,
}

pub async fn find(
    info: web::Query<FindQueryParameters>,
    cs: web::Data<CorpusStorage>,
    db_pool: web::Data<DbPool>,
    claims: ClaimsAuth,
) -> Result<HttpResponse, ServiceError> {
    let order = if let Some(order) = &info.order {
        match order.to_lowercase().as_str() {
            "ascending" => ResultOrder::Normal,
            "random" => ResultOrder::Randomized,
            "descending" => ResultOrder::Inverted,
            "unsorted" => ResultOrder::NotSorted,
            _ => ResultOrder::Normal,
        }
    } else {
        ResultOrder::Normal
    };
    let corpora = parse_corpora(&info.corpora);
    if corpus_access_allowed(corpora.clone(), claims, db_pool).await? {
        let matches = cs
            .find(
                &corpora,
                &info.q,
                parse_query_language(&info.query_language),
                info.offset.unwrap_or_default(),
                info.limit,
                order,
            )
            .unwrap();
        Ok(HttpResponse::Ok().body(matches.join("\n")))
    } else {
        Err(ServiceError::NonAuthorizedCorpus)
    }
}

#[derive(Deserialize)]
pub struct CountQueryParameters {
    q: String,
    #[serde(default)]
    query_language: Option<String>,
    corpora: String,
}

pub async fn count(
    info: web::Query<CountQueryParameters>,
    cs: web::Data<CorpusStorage>,
    db_pool: web::Data<DbPool>,
    claims: ClaimsAuth,
) -> Result<HttpResponse, ServiceError> {
    let corpora = parse_corpora(&info.corpora);
    if corpus_access_allowed(corpora.clone(), claims, db_pool).await? {
        let count = cs
            .count(
                &corpora,
                &info.q,
                parse_query_language(&info.query_language),
            )
            .unwrap();
        Ok(HttpResponse::Ok().body(format!("{}", count)))
    } else {
        Err(ServiceError::NonAuthorizedCorpus)
    }
}
