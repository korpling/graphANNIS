use std::vec;

use crate::auth::Claims;

use super::*;
use actix_web::{
    body::to_bytes,
    http,
    web::{Bytes, Path},
};
use diesel::{r2d2::ConnectionManager, SqliteConnection};
use diesel_migrations::MigrationHarness;

use pretty_assertions::assert_eq;

trait BodyTest {
    fn as_str(&self) -> &str;
}

impl BodyTest for Bytes {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}

fn create_empty_dbpool() -> r2d2::Pool<ConnectionManager<SqliteConnection>> {
    let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
    let db_pool = r2d2::Pool::builder().build(manager).unwrap();
    let mut conn = db_pool.get().unwrap();
    conn.run_pending_migrations(crate::MIGRATIONS).unwrap();

    db_pool
}

#[actix_web::test]
async fn test_user_groups() {
    // Declare a admin user
    let admin_claims = ClaimsFromAuth(Claims {
        sub: "admin".to_string(),
        exp: None,
        roles: vec!["admin".to_string()],
        groups: vec![],
    });
    // Init temporary database
    let db_pool = create_empty_dbpool();
    let db_pool = web::Data::new(db_pool);

    // Initial list of groups, this should be empty
    let resp = list_groups(db_pool.clone(), admin_claims.clone())
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body.as_str(), "[]");

    // Add a new group and check it has been persisted
    let request_body = web::Json(Group {
        name: "academic".to_string(),
        corpora: vec!["pcc2".to_string(), "GUM".to_string()],
    });
    let resp = put_group(
        Path::from("academic".to_string()),
        request_body,
        db_pool.clone(),
        admin_claims.clone(),
    )
    .await
    .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);

    let resp = list_groups(db_pool.clone(), admin_claims.clone())
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(
        body.as_str(),
        "[{\"name\":\"academic\",\"corpora\":[\"GUM\",\"pcc2\"]}]"
    );

    // Delete group again and check it has been removed
    let resp = delete_group(
        Path::from("academic".to_string()),
        db_pool.clone(),
        admin_claims.clone(),
    )
    .await
    .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
    let resp = list_groups(db_pool.clone(), admin_claims.clone())
        .await
        .unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body.as_str(), "[]");
}
