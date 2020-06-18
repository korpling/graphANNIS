use super::check_corpora_authorized;
use crate::{
    actions, errors::ServiceError, extractors::ClaimsFromAuth, settings::Settings, DbPool,
};
use actix_files::NamedFile;
use actix_web::web::{self, HttpResponse};
use graphannis::{
    corpusstorage::QueryLanguage, graph, model::AnnotationComponentType, CorpusStorage,
};
use std::path::PathBuf;

pub async fn list(
    cs: web::Data<CorpusStorage>,
    claims: ClaimsFromAuth,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, ServiceError> {
    let all_corpora: Vec<String> = cs.list()?.into_iter().map(|c| c.name).collect();

    let allowed_corpora = if claims.0.admin {
        // Adminstrators always have access to all corpora
        all_corpora
    } else {
        // Query the database for all allowed corpora of this user
        let conn = db_pool.get()?;
        let corpora_by_group =
            web::block(move || actions::authorized_corpora_from_groups(&claims.0, &conn)).await?;
        // Filter out non-existing corpora
        all_corpora
            .into_iter()
            .filter(|c| corpora_by_group.contains(c))
            .collect()
    };

    Ok(HttpResponse::Ok().json(allowed_corpora))
}

#[derive(Deserialize)]
pub struct SubgraphWithContext {
    node_ids: Vec<String>,
    #[serde(default)]
    segmentation: Option<String>,
    #[serde(default)]
    left: usize,
    #[serde(default)]
    right: usize,
}

pub async fn subgraph(
    corpus: web::Path<String>,
    params: web::Json<SubgraphWithContext>,
    cs: web::Data<CorpusStorage>,
    db_pool: web::Data<DbPool>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    check_corpora_authorized(vec![corpus.clone()], claims.0, &db_pool).await?;
    let graph = cs.subgraph(
        &corpus,
        params.node_ids.clone(),
        params.left,
        params.right,
        params.segmentation.clone(),
    )?;
    // Export subgraph to GraphML
    let mut output = Vec::new();
    graphannis_core::graph::serialization::graphml::export(&graph, None, &mut output, |_| {})?;

    Ok(HttpResponse::Ok()
        .content_type("application/xml")
        .body(output))
}

#[derive(Deserialize)]
pub struct QuerySubgraphParameters {
    query: String,
    #[serde(default)]
    query_language: QueryLanguage,
    #[serde(default)]
    component_type_filter: Option<AnnotationComponentType>,
}

pub async fn subgraph_for_query(
    corpus: web::Path<String>,
    params: web::Query<QuerySubgraphParameters>,
    cs: web::Data<CorpusStorage>,
    db_pool: web::Data<DbPool>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    check_corpora_authorized(vec![corpus.clone()], claims.0, &db_pool).await?;

    let graph = cs.subgraph_for_query(
        &corpus,
        params.query.as_str(),
        params.query_language,
        params.component_type_filter.clone(),
    )?;
    // Export subgraph to GraphML
    let mut output = Vec::new();
    graphannis_core::graph::serialization::graphml::export(&graph, None, &mut output, |_| {})?;

    Ok(HttpResponse::Ok()
        .content_type("application/xml")
        .body(output))
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
pub struct AnnotationParameters {
    #[serde(default)]
    list_values: bool,
    #[serde(default)]
    only_most_frequent_values: bool,
}

pub async fn node_annotations(
    corpus: web::Path<String>,
    params: web::Query<AnnotationParameters>,
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
pub async fn edge_annotations(
    path: web::Path<(String, AnnotationComponentType, String, String)>,
    params: web::Query<AnnotationParameters>,
    cs: web::Data<CorpusStorage>,
    claims: ClaimsFromAuth,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, ServiceError> {
    let (corpus, ctype, layer, name) = path.as_ref();
    check_corpora_authorized(vec![corpus.clone()], claims.0, &db_pool).await?;

    let component = graph::Component::<AnnotationComponentType>::new(
        ctype.to_owned(),
        layer.to_string(),
        name.to_string(),
    );

    let annos = cs.list_edge_annotations(
        corpus.as_str(),
        &component,
        params.list_values,
        params.only_most_frequent_values,
    );

    Ok(HttpResponse::Ok().json(annos))
}

#[derive(Deserialize)]
pub struct FilesParameters {
    name: String,
}

pub async fn files(
    corpus: web::Path<String>,
    params: web::Query<FilesParameters>,
    claims: ClaimsFromAuth,
    db_pool: web::Data<DbPool>,
    settings: web::Data<Settings>,
) -> Result<NamedFile, ServiceError> {
    check_corpora_authorized(vec![corpus.clone()], claims.0, &db_pool).await?;

    // Perform some sanity checks to make sure only the relative sub-folder is used
    let file_path = params.name.trim();
    if file_path.contains("..") {
        return Err(ServiceError::BadRequest(
            "No .. allowed in file name".to_string(),
        ));
    } else if file_path.starts_with("/") {
        return Err(ServiceError::BadRequest(
            "No absolute path allowed in file name".to_string(),
        ));
    }

    // Resolve against data folder
    let path = PathBuf::from(settings.database.graphannis.as_str())
        .join(corpus.as_str())
        .join("files")
        .join(&file_path);

    Ok(NamedFile::open(path)?)
}
