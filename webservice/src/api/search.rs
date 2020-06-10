use crate::{actions, errors::ServiceError, AppState};
use actix_web::web::{self, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use graphannis::corpusstorage::{QueryLanguage, ResultOrder};
use serde::Deserialize;

async fn corpus_access_allowed(
    requested_corpora: Vec<String>,
    credentials: BearerAuth,
    state: &web::Data<AppState>,
) -> Result<bool, ServiceError> {
    let conn = state
        .sqlite_pool
        .get()
        .map_err(|_| ServiceError::DatabaseError)?;
    let jwt_secret = state.settings.auth.jwt_secret.clone();
    let access_allowed: bool = web::block(move || {
        actions::corpus_access_allowed(&requested_corpora, credentials.token(), &jwt_secret, &conn)
    })
    .await
    .map_err(|_| ServiceError::InternalServerError)?;
    Ok(access_allowed)
}

fn parse_query_language(query_language: &Option<String>) -> QueryLanguage {
    if let Some(query_language) = query_language {
        if query_language.to_lowercase() == "aqlquirksv3" {
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
    state: web::Data<AppState>,
    credentials: BearerAuth,
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
    if corpus_access_allowed(corpora.clone(), credentials, &state).await? {
        let matches = state
            .cs
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
    state: web::Data<AppState>,
    credentials: BearerAuth,
) -> Result<HttpResponse, ServiceError> {
    let corpora = parse_corpora(&info.corpora);
    if corpus_access_allowed(corpora.clone(), credentials, &state).await? {
        let count = state
            .cs
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
