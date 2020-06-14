use super::{check_corpora_authorized, parse_corpora, parse_query_language};
use crate::{errors::ServiceError, extractors::ClaimsFromAuth, DbPool};
use actix_web::web::{self, HttpResponse};
use graphannis::{corpusstorage::ResultOrder, CorpusStorage};
use serde::Deserialize;

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
    params: web::Query<FindQueryParameters>,
    cs: web::Data<CorpusStorage>,
    db_pool: web::Data<DbPool>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    let corpora =
        check_corpora_authorized(parse_corpora(&params.corpora), claims.0, &db_pool).await?;

    let order = if let Some(order) = &params.order {
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

    let matches = cs.find(
        &corpora,
        &params.q,
        parse_query_language(&params.query_language),
        params.offset.unwrap_or_default(),
        params.limit,
        order,
    )?;
    Ok(HttpResponse::Ok().body(matches.join("\n")))
}

#[derive(Deserialize)]
pub struct CountQueryParameters {
    q: String,
    #[serde(default)]
    query_language: Option<String>,
    corpora: String,
}

pub async fn count(
    params: web::Query<CountQueryParameters>,
    cs: web::Data<CorpusStorage>,
    db_pool: web::Data<DbPool>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    let corpora =
        check_corpora_authorized(parse_corpora(&params.corpora), claims.0, &db_pool).await?;
    let count = cs.count(
        &corpora,
        &params.q,
        parse_query_language(&params.query_language),
    )?;
    Ok(HttpResponse::Ok().body(format!("{}", count)))
}
