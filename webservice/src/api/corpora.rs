use super::{check_corpora_authorized, check_is_admin};
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


#[derive(Deserialize)]
pub struct SubgraphQueryParameters {
    node_ids: String,
    #[serde(default)]
    segmentation: Option<String>,
    #[serde(default)]
    left: usize,
    #[serde(default)]
    right: usize,
}

pub async fn subgraph(
    corpus: web::Path<String>,
    params: web::Query<SubgraphQueryParameters>,
    cs: web::Data<CorpusStorage>,
    db_pool: web::Data<DbPool>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    check_corpora_authorized(vec![corpus.clone()], claims.0, &db_pool).await?;
    let node_ids: Vec<String> = params
        .node_ids
        .split(",")
        .map(|c| c.trim().to_string())
        .collect();
    let graph = cs.subgraph(
        &corpus,
        node_ids,
        params.left,
        params.right,
        params.segmentation.clone(),
    )?;
    // Export subgraph to GraphML
    let mut output = Vec::new();
    graphannis_core::graph::serialization::graphml::export(&graph, &mut output, |_| {})?;

    Ok(HttpResponse::Ok().body(output))
}


pub async fn configuration(
    corpus: web::Path<String>,
    cs: web::Data<CorpusStorage>,
    claims: ClaimsFromAuth,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, ServiceError> {
    check_corpora_authorized(vec![corpus.clone()], claims.0, &db_pool).await?;

    let corpus_info = cs.info(corpus.as_str())?;
    
    Ok(HttpResponse::Ok().json(corpus_info.config))
}

