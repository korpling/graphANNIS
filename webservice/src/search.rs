use actix_web::{get, web};
use serde::Deserialize;

#[derive(Deserialize)]
struct QueryParameters {
    q: String,
    query_language: String,
    corpora: String,
}

#[get("/search/count")]
async fn count(info: web::Query::<QueryParameters>) ->  String {
    format!("Querying corpus {:?} with query '{}' and language {}", info.corpora, info.q, info.query_language)
}