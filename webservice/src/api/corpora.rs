use super::{check_corpora_authorized, check_is_admin};
use crate::{
    actions, errors::ServiceError, extractors::ClaimsFromAuth, settings::Settings, DbPool,
};
use actix_files::NamedFile;
use actix_web::web::{self, HttpResponse};
use graphannis::{graph, model::AnnotationComponentType, CorpusStorage};
use std::path::PathBuf;

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

#[derive(Deserialize, Clone)]
pub struct ListComponentsParameters {
    #[serde(rename = "type")]
    ctype: Option<AnnotationComponentType>,
    name: Option<String>,
}

#[derive(Serialize)]
pub struct Component {
    /// Type of the component
    #[serde(rename = "type")]
    ctype: AnnotationComponentType,
    /// Name of the component
    name: String,
    /// A layer name which allows to group different components into the same layer. Can be empty.
    layer: String,
}

pub async fn list_components(
    corpus: web::Path<String>,
    params: web::Query<ListComponentsParameters>,
    cs: web::Data<CorpusStorage>,
    claims: ClaimsFromAuth,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, ServiceError> {
    check_corpora_authorized(vec![corpus.clone()], claims.0, &db_pool).await?;

    let components: Vec<_> = cs
        .list_components(
            corpus.as_str(),
            params.clone().ctype,
            params.name.as_deref(),
        )
        .into_iter()
        .map(|c| Component {
            ctype: c.get_type(),
            name: c.name,
            layer: c.layer,
        })
        .collect();

    Ok(HttpResponse::Ok().json(components))
}

#[derive(Deserialize)]
pub struct NodeAnnotationParameters {
    #[serde(default)]
    list_values: bool,
    #[serde(default)]
    only_most_frequent_values: bool,
}

pub async fn node_annotations(
    corpus: web::Path<String>,
    params: web::Query<NodeAnnotationParameters>,
    cs: web::Data<CorpusStorage>,
    claims: ClaimsFromAuth,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, ServiceError> {
    check_corpora_authorized(vec![corpus.clone()], claims.0, &db_pool).await?;

    let annos = cs.list_node_annotations(
        corpus.as_str(),
        params.list_values,
        params.only_most_frequent_values,
    );

    Ok(HttpResponse::Ok().json(annos))
}

#[derive(Deserialize)]
pub struct EdgeAnnotationParameters {
    #[serde(rename = "type")]
    ctype: AnnotationComponentType,
    name: String,
    layer: String,
    #[serde(default)]
    list_values: bool,
    #[serde(default)]
    only_most_frequent_values: bool,
}

pub async fn edge_annotations(
    corpus: web::Path<String>,
    params: web::Query<EdgeAnnotationParameters>,
    cs: web::Data<CorpusStorage>,
    claims: ClaimsFromAuth,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, ServiceError> {
    check_corpora_authorized(vec![corpus.clone()], claims.0, &db_pool).await?;

    let component = graph::Component::<AnnotationComponentType>::new(
        params.ctype.to_owned(),
        params.layer.to_string(),
        params.name.to_string(),
    );

    let annos = cs.list_edge_annotations(
        corpus.as_str(),
        &component,
        params.list_values,
        params.only_most_frequent_values,
    );

    Ok(HttpResponse::Ok().json(annos))
}

pub async fn files(
    path: web::Path<(String, String)>,
    claims: ClaimsFromAuth,
    db_pool: web::Data<DbPool>,
    settings: web::Data<Settings>,
) -> Result<NamedFile, ServiceError> {
    let corpus = path.0.clone();
    let file_path = path.1.clone();
    check_corpora_authorized(vec![corpus.clone()], claims.0, &db_pool).await?;

    // Resolve against data folder
    let path = PathBuf::from(settings.database.graphannis.as_str())
        .join(corpus)
        .join("files")
        .join(file_path);

    Ok(NamedFile::open(path)?)
}
