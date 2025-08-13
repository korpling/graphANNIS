use super::check_is_admin;
use crate::{
    DbPool, actions, errors::ServiceError, extractors::ClaimsFromAuth, settings::Settings,
};
use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, web};
use futures::prelude::*;
use graphannis::CorpusStorage;
use std::io::Seek;
use std::{collections::HashMap, fs::File, io::Write, sync::Mutex};

#[derive(Serialize, Deserialize, Clone)]
pub struct Group {
    pub name: String,
    pub corpora: Vec<String>,
}

#[derive(Serialize)]
pub enum JobStatus {
    Running,
    Failed,
    #[serde(skip)]
    Finished(Option<(File, String)>),
}

#[derive(Serialize)]
pub enum JobType {
    Import,
    Export,
}

#[derive(Serialize)]
pub struct Job {
    job_type: JobType,
    messages: Vec<String>,
    status: JobStatus,
}

#[derive(Default)]
pub struct BackgroundJobs {
    // Maps a UUID to a job
    pub jobs: Mutex<HashMap<uuid::Uuid, Job>>,
}

pub async fn list_groups(
    db_pool: web::Data<DbPool>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    check_is_admin(&claims.0)?;

    let mut conn = db_pool.get()?;
    let corpus_groups = web::block::<_, Result<_, ServiceError>>(move || {
        let result = actions::list_groups(&mut conn)?;
        Ok(result)
    })
    .await??;

    Ok(HttpResponse::Ok().json(corpus_groups))
}

pub async fn delete_group(
    group_name: web::Path<String>,
    db_pool: web::Data<DbPool>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    check_is_admin(&claims.0)?;

    let mut conn = db_pool.get()?;
    web::block::<_, Result<_, ServiceError>>(move || actions::delete_group(&group_name, &mut conn))
        .await??;

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

    let mut conn = db_pool.get()?;
    web::block::<_, Result<_, ServiceError>>(move || {
        actions::add_or_replace_group(group.clone(), &mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json("Group added/replaced"))
}

#[derive(Deserialize, Clone)]
pub struct ImportParams {
    #[serde(default)]
    override_existing: bool,
}

#[derive(Serialize)]
pub struct JobReference {
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
        tmp = web::block(move || tmp.write_all(&data).map(|_| tmp)).await??;
    }

    // Create a UUID which is used for the background job
    let id = uuid::Uuid::new_v4();
    {
        let mut jobs = background_jobs.jobs.lock()?;
        jobs.insert(
            id,
            Job {
                job_type: JobType::Import,
                messages: Vec::default(),
                status: JobStatus::Running,
            },
        );
    }
    // Execute the whole import in a background thread
    std::thread::spawn(move || {
        let id_as_string = id.to_string();
        let import_result = cs.import_all_from_zip(
            tmp,
            settings.database.disk_based,
            params.override_existing,
            |status| {
                info!("Job {} update: {}", &id_as_string, status);
                // Add status report to background job messages
                if let Ok(mut jobs) = background_jobs.jobs.lock()
                    && let Some(j) = jobs.get_mut(&id)
                {
                    j.messages.push(status.to_string());
                }
            },
        );
        match import_result {
            Ok(corpora) => {
                if let Ok(mut jobs) = background_jobs.jobs.lock()
                    && let Some(j) = jobs.get_mut(&id)
                {
                    j.messages.push(format!("imported corpora {:?}", corpora));
                    j.status = JobStatus::Finished(None);
                }
            }
            Err(err) => {
                if let Ok(mut jobs) = background_jobs.jobs.lock()
                    && let Some(j) = jobs.get_mut(&id)
                {
                    j.messages
                        .push(format!("importing corpora failed: {:?}", err));
                    j.status = JobStatus::Failed;
                }
            }
        }
    });

    Ok(HttpResponse::Accepted().json(JobReference {
        uuid: id.to_string(),
    }))
}

#[derive(Deserialize, Serialize)]
pub struct ExportParams {
    corpora: Vec<String>,
}

fn export_corpus_background_taks(
    corpora: &[String],
    cs: &CorpusStorage,
    id: uuid::Uuid,
    background_jobs: web::Data<BackgroundJobs>,
) -> Result<File, ServiceError> {
    // Create temporary file to export to. We can't use the ZipArchive with the
    // response body because it does implement `Write` but not `Seek`.
    let tmp_zip = tempfile::tempfile()?;

    let mut zip = zip::ZipWriter::new(tmp_zip);

    let id_as_string = id.to_string();

    let use_corpus_subdirectory = corpora.len() > 1;
    for corpus_name in corpora {
        // Add the GraphML file to the ZIP file
        let corpus_name: &str = corpus_name.as_ref();
        cs.export_to_zip(corpus_name, use_corpus_subdirectory, &mut zip, |status| {
            info!("Job {} update: {}", &id_as_string, status);
            // Add status report to background job messages
            if let Ok(mut jobs) = background_jobs.jobs.lock()
                && let Some(j) = jobs.get_mut(&id)
            {
                j.messages.push(status.to_string());
            }
        })?;
    }
    let mut tmp_zip = zip.finish()?;
    tmp_zip.seek(std::io::SeekFrom::Start(0))?;
    Ok(tmp_zip)
}

pub async fn export_corpus(
    params: web::Json<ExportParams>,
    cs: web::Data<CorpusStorage>,
    claims: ClaimsFromAuth,
    background_jobs: web::Data<BackgroundJobs>,
) -> Result<HttpResponse, ServiceError> {
    check_is_admin(&claims.0)?;

    // Create a UUID which is used for the background job
    let id = uuid::Uuid::new_v4();
    {
        let mut jobs = background_jobs.jobs.lock()?;
        jobs.insert(
            id,
            Job {
                job_type: JobType::Export,
                messages: Vec::default(),
                status: JobStatus::Running,
            },
        );
    }
    // Execute the whole import in a background thread
    std::thread::spawn(move || {
        match export_corpus_background_taks(&params.corpora, &cs, id, background_jobs.clone()) {
            Ok(tmp_file) => {
                if let Ok(mut jobs) = background_jobs.jobs.lock()
                    && let Some(j) = jobs.get_mut(&id)
                {
                    let created_file_name = params.corpora.join("_") + ".zip";
                    j.status = JobStatus::Finished(Some((tmp_file, created_file_name)));
                }
            }
            Err(err) => {
                if let Ok(mut jobs) = background_jobs.jobs.lock()
                    && let Some(j) = jobs.get_mut(&id)
                {
                    j.messages
                        .push(format!("exporting corpora failed: {:?}", err));
                    j.status = JobStatus::Failed;
                }
            }
        }
    });

    Ok(HttpResponse::Accepted().json(JobReference {
        uuid: id.to_string(),
    }))
}

pub async fn jobs(
    uuid: web::Path<String>,
    background_jobs: web::Data<BackgroundJobs>,
    claims: ClaimsFromAuth,
    req: HttpRequest,
) -> Result<HttpResponse, ServiceError> {
    check_is_admin(&claims.0)?;

    let uuid = uuid::Uuid::parse_str(&uuid)?;

    let mut jobs = background_jobs.jobs.lock()?;
    if let Some(j) = jobs.get(&uuid)
        && let JobStatus::Running = j.status
    {
        // Job still running, do not remove it from the job list
        return Ok(HttpResponse::Accepted().json(j));
    }
    // Job is finished/errored: remove it from the list and process it
    if let Some(j) = jobs.remove(&uuid) {
        match j.status {
            JobStatus::Failed => {
                return Ok(HttpResponse::Gone().json(j));
            }
            JobStatus::Finished(result) => {
                if let Some((tmp_file, file_name)) = result {
                    let named_file = NamedFile::from_file(tmp_file, file_name)?;
                    let response = named_file.into_response(&req);
                    return Ok(response);
                } else {
                    return Ok(HttpResponse::Ok().json(j.messages));
                }
            }
            _ => {}
        }
    }
    Ok(HttpResponse::NotFound().finish())
}

#[cfg(test)]
mod tests;
