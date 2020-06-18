use super::check_is_admin;
use crate::{
    actions, errors::ServiceError, extractors::ClaimsFromAuth, settings::Settings, DbPool,
};
use actix_web::web::{self, HttpResponse};
use futures::prelude::*;
use graphannis::CorpusStorage;
use std::io::Write;

#[derive(Serialize, Deserialize, Clone)]
pub struct Group {
    pub name: String,
    pub corpora: Vec<String>,
}

pub async fn list_groups(
    db_pool: web::Data<DbPool>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    check_is_admin(&claims.0)?;

    let conn = db_pool.get()?;
    let corpus_groups = web::block::<_, _, ServiceError>(move || {
        let result = actions::list_groups(&conn)?;
        Ok(result)
    })
    .await?;

    Ok(HttpResponse::Ok().json(corpus_groups))
}

pub async fn delete_group(
    group_name: web::Path<String>,
    db_pool: web::Data<DbPool>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    check_is_admin(&claims.0)?;

    let conn = db_pool.get()?;
    web::block::<_, _, ServiceError>(move || actions::delete_group(&group_name, &conn)).await?;

    Ok(HttpResponse::Ok().json("Group deleted"))
}

pub async fn put_group(
    group_name: web::Path<String>,
    group: web::Json<Group>,
    db_pool: web::Data<DbPool>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    check_is_admin(&claims.0)?;

    if group_name.as_str() != group.name.as_str() {
        return Ok(HttpResponse::BadRequest().json("Group name in path and object need to match."));
    }

    let conn = db_pool.get()?;
    web::block::<_, _, ServiceError>(move || actions::add_or_replace_group(group.clone(), &conn))
        .await?;

    Ok(HttpResponse::Ok().json("Group added/replaced"))
}

#[derive(Deserialize, Clone)]
pub struct ImportParams {
    #[serde(default)]
    override_existing: bool,
}

pub async fn import_corpus(
    params: web::Query<ImportParams>,
    mut body: web::Payload,
    cs: web::Data<CorpusStorage>,
    settings: web::Data<Settings>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    check_is_admin(&claims.0)?;

    // Copy the request body, which should be a ZIP file, to a temporary file
    let mut tmp = tempfile::tempfile()?;
    while let Some(chunk) = body.next().await {
        let data = chunk?;
        tmp = web::block(move || tmp.write_all(&data).map(|_| tmp)).await?;
    }

    cs.import_all_from_zip(tmp, settings.database.disk_based, params.override_existing)?;

    Ok(HttpResponse::Ok().json("Corpus imported"))
}
