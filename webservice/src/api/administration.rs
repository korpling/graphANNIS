use super::check_is_admin;
use crate::{actions, errors::ServiceError, extractors::ClaimsFromAuth, DbPool};
use actix_web::web::{self, HttpResponse};

#[derive(Serialize, Deserialize)]
pub struct CorpusGroup {
    name: String,
    corpora: Vec<String>,
}

pub async fn list_groups(
    db_pool: web::Data<DbPool>,
    claims: ClaimsFromAuth,
) -> Result<HttpResponse, ServiceError> {
    check_is_admin(&claims.0)?;

    let conn = db_pool.get()?;
    let corpus_groups = web::block::<_, _, ServiceError>(move || {
        let mut result: Vec<CorpusGroup> = Vec::new();
        // Collect the corpora for each group name
        for group_name in actions::get_group_names(&conn)? {
            result.push(CorpusGroup {
                corpora: actions::get_corpora_for_group(&group_name, &conn)?,
                name: group_name,
            })
        }

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
