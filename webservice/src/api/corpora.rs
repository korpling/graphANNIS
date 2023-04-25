use super::{check_corpora_authorized_read, check_is_admin};
use crate::{
    actions, errors::ServiceError, extractors::ClaimsFromAuth, settings::Settings, DbPool,
};
use actix_files::NamedFile;
use actix_web::{web, HttpResponse};
use graphannis::{
    corpusstorage::QueryLanguage, graph, model::AnnotationComponentType, CorpusStorage,
};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::{borrow::Cow, path::PathBuf};

pub const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    .add(b'%')
    .add(b'/');

pub async fn list(
    cs: web::Data<CorpusStorage>,
    claims: ClaimsFromAuth,
    db_pool: web::Data<DbPool>,
    settings: web::Data<Settings>,
) -> Result<HttpResponse, ServiceError> {
    let all_corpora: Vec<String> = cs.list()?.into_iter().map(|c| c.name).collect();

    let mut allowed_corpora = if claims.0.roles.iter().any(|r| r.as_str() == "admin")
        || settings.auth.anonymous_access_all_corpora
    {
        // Administrators always have access to all corpora or read-access is
        // configured to be granted without login
        all_corpora
    } else {
        // Query the database for all allowed corpora of this user
        let mut conn = db_pool.get()?;
        let corpora_by_group =
            web::block(move || actions::authorized_corpora_from_groups(&claims.0, &mut conn))
                .await??;
        // Filter out non-existing corpora
        all_corpora
            .into_iter()
            .filter(|c| corpora_by_group.contains(c))
            .collect()
    };

    allowed_corpora.sort_unstable();

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
    settings: web::Data<Settings>,
) -> Result<HttpResponse, ServiceError> {
    check_corpora_authorized_read(vec![corpus.clone()], claims.0, &settings, &db_pool).await?;
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
    settings: web::Data<Settings>,
) -> Result<HttpResponse, ServiceError> {
    check_corpora_authorized_read(vec![corpus.clone()], claims.0, &settings, &db_pool).await?;

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
    settings: web::Data<Settings>,
) -> Result<HttpResponse, ServiceError> {
    check_corpora_authorized_read(vec![corpus.clone()], claims.0, &settings, &db_pool).await?;

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
    settings: web::Data<Settings>,
) -> Result<HttpResponse, ServiceError> {
    check_corpora_authorized_read(vec![corpus.clone()], claims.0, &settings, &db_pool).await?;

    let components: Vec<_> = cs
        .list_components(
            corpus.as_str(),
            params.ctype.clone(),
            params.name.as_deref(),
        )?
        .into_iter()
        .map(|c| Component {
            ctype: c.get_type(),
            name: c.name.into(),
            layer: c.layer.into(),
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
    settings: web::Data<Settings>,
) -> Result<HttpResponse, ServiceError> {
    check_corpora_authorized_read(vec![corpus.clone()], claims.0, &settings, &db_pool).await?;

    let annos = cs.list_node_annotations(
        corpus.as_str(),
        params.list_values,
        params.only_most_frequent_values,
    )?;

    Ok(HttpResponse::Ok().json(annos))
}
pub async fn edge_annotations(
    path: web::Path<(String, AnnotationComponentType, String, String)>,
    params: web::Query<AnnotationParameters>,
    cs: web::Data<CorpusStorage>,
    claims: ClaimsFromAuth,
    db_pool: web::Data<DbPool>,
    settings: web::Data<Settings>,
) -> Result<HttpResponse, ServiceError> {
    let (corpus, ctype, layer, name) = path.as_ref();
    check_corpora_authorized_read(vec![corpus.clone()], claims.0, &settings, &db_pool).await?;

    let component = graph::Component::<AnnotationComponentType>::new(
        ctype.to_owned(),
        layer.into(),
        name.into(),
    );

    let annos = cs.list_edge_annotations(
        corpus.as_str(),
        &component,
        params.list_values,
        params.only_most_frequent_values,
    )?;

    Ok(HttpResponse::Ok().json(annos))
}

#[derive(Deserialize)]
pub struct ListFilesParameters {
    node: Option<String>,
}

pub async fn list_files(
    corpus: web::Path<String>,
    params: web::Query<ListFilesParameters>,
    claims: ClaimsFromAuth,
    db_pool: web::Data<DbPool>,
    settings: web::Data<Settings>,
) -> Result<HttpResponse, ServiceError> {
    check_corpora_authorized_read(vec![corpus.clone()], claims.0, &settings, &db_pool).await?;

    let mut found_files = Vec::default();
    let escaped_corpus_name: Cow<str> =
        utf8_percent_encode(&corpus, PATH_SEGMENT_ENCODE_SET).into();
    // Get the base path
    let base_path = PathBuf::from(settings.database.graphannis.as_str())
        .join(escaped_corpus_name.to_string())
        .join("files")
        .canonicalize()?;

    // if the node name is set, restrict the search to the subfolder with the same ID, otherwise use the corpus ID
    let search_path = if let Some(node) = &params.node {
        // Perform some sanity checks to make sure only the relative sub-folder is used
        let file_path = node.trim();
        if file_path.contains("..") {
            return Err(ServiceError::IllegalNodePath(
                "No .. allowed in file name".to_string(),
            ));
        } else if file_path.starts_with('/') {
            return Err(ServiceError::IllegalNodePath(
                "No absolute path allowed in file name".to_string(),
            ));
        };
        base_path.join(file_path)
    } else {
        base_path.join(corpus.as_ref())
    };

    // List all files in the search path
    if search_path.exists() && search_path.is_dir() {
        for entry in walkdir::WalkDir::new(search_path) {
            let entry = entry?;
            // get the absolute path
            let entry_path = entry.path().canonicalize()?;
            if entry_path.is_file() {
                // get relative path to base path and add it to result
                let resolved_path = entry_path.strip_prefix(&base_path)?;
                found_files.push(resolved_path.to_string_lossy().to_string());
            }
        }
    }

    Ok(HttpResponse::Ok().json(found_files))
}

pub async fn file_content(
    path: web::Path<(String, String)>,
    claims: ClaimsFromAuth,
    db_pool: web::Data<DbPool>,
    settings: web::Data<Settings>,
) -> Result<NamedFile, ServiceError> {
    let (corpus, name) = path.into_inner();
    let name = percent_encoding::percent_decode_str(&name).decode_utf8_lossy();

    check_corpora_authorized_read(vec![corpus.clone()], claims.0, &settings, &db_pool).await?;

    // Perform some sanity checks to make sure only the relative sub-folder is used
    let file_path = name.trim();
    if file_path.contains("..") {
        return Err(ServiceError::IllegalNodePath(
            "No .. allowed in file name".to_string(),
        ));
    } else if file_path.starts_with('/') {
        return Err(ServiceError::IllegalNodePath(
            "No absolute path allowed in file name".to_string(),
        ));
    }

    // Resolve against data folder
    let escaped_corpus_name: Cow<str> =
        utf8_percent_encode(&corpus, PATH_SEGMENT_ENCODE_SET).into();
    let path = PathBuf::from(settings.database.graphannis.as_str())
        .join(escaped_corpus_name.to_string())
        .join("files")
        .join(file_path);

    Ok(NamedFile::open(path)?)
}
pub async fn delete(
    path: web::Path<String>,
    claims: ClaimsFromAuth,
    cs: web::Data<CorpusStorage>,
) -> Result<HttpResponse, ServiceError> {
    check_is_admin(&claims.0)?;

    if cs.delete(path.as_ref())? {
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

#[cfg(test)]
mod tests;
