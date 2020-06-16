use super::check_is_admin;
use crate::{actions, errors::ServiceError, extractors::ClaimsFromAuth, DbPool};
use actix_web::web::{self, HttpResponse};

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
