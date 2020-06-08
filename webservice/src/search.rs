use actix_web::{get, web};
use serde::Deserialize;
use crate::AppState;

#[derive(Deserialize)]
struct QueryParameters {
    q: String,
    query_language: String,
    corpora: String,
}

#[get("/search/count")]
async fn count(info: web::Query::<QueryParameters>, state: web::Data<AppState>) ->  String {
    let corpora = vec![info.corpora.clone()];
    let count = state.cs.count(&corpora, &info.q, graphannis::corpusstorage::QueryLanguage::AQL).unwrap();
    format!("{}", count)
}