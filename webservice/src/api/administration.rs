use super::check_is_admin;
use crate::{
    actions, errors::ServiceError, extractors::ClaimsFromAuth, settings::Settings, DbPool,
};
use actix_web::web::{self, HttpResponse};
use futures::prelude::*;
use graphannis::CorpusStorage;
use std::{collections::HashMap, io::Write, sync::RwLock};

#[derive(Serialize, Deserialize, Clone)]
pub struct Group {
    pub name: String,
    pub corpora: Vec<String>,
}

#[derive(Serialize, Clone)]
pub enum Job {
    Import { messages: Vec<String> },
}

#[derive(Default)]
pub struct BackgroundJobs {
    // Maps a UUID to a job
    pub jobs: RwLock<HashMap<uuid::Uuid, Job>>,
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

#[derive(Serialize)]
pub struct ImportResult {
    uuid: String,
}

pub async fn import_corpus(
    params: web::Query<ImportParams>,
    mut body: web::Payload,
    background_jobs: web::Data<BackgroundJobs>,
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

    // Create a UUID which is used for the background job
    let id = uuid::Uuid::new_v4();
    {
        let mut jobs = background_jobs.jobs.write().expect("Lock was poisoned");
        jobs.insert(
            id,
            Job::Import {
                messages: Vec::default(),
            },
        );
    }
    // Execute the whole import in a background thread
    std::thread::spawn(move || {
        let id_as_string = id.to_string();
        match cs.import_all_from_zip(
            tmp,
            settings.database.disk_based,
            params.override_existing,
            |status| {
                info!("Job {} update: {}", &id_as_string, status);
                // Add status report to background job messages
                let mut jobs = background_jobs.jobs.write().expect("Lock was poisoned");
                if let Some(Job::Import { messages }) = jobs.get_mut(&id) {
                    messages.push(status.to_string());
                }
            },
        ) {
            Ok(_corpora) => {}
            Err(_err) => {}
        }
    });

    // TODO: calculate the correct endpoint using the ID
    Ok(HttpResponse::Accepted().json(ImportResult {
        uuid: id.to_string(),
    }))
}
