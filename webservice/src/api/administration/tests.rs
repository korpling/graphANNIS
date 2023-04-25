use std::{
    time::{Duration, SystemTime, UNIX_EPOCH},
    vec,
};

use crate::{
    auth::Claims,
    settings::{JWTVerification, Settings},
};

use super::*;
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    http::{self, StatusCode},
    test, App,
};
use diesel::{r2d2::ConnectionManager, SqliteConnection};
use diesel_migrations::MigrationHarness;

use jsonwebtoken::EncodingKey;
use pretty_assertions::assert_eq;

const JWT_SECRET: &str = "not-a-secret";

fn create_empty_dbpool() -> r2d2::Pool<ConnectionManager<SqliteConnection>> {
    let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
    let db_pool = r2d2::Pool::builder().build(manager).unwrap();
    let mut conn = db_pool.get().unwrap();
    conn.run_pending_migrations(crate::MIGRATIONS).unwrap();

    db_pool
}

fn create_test_app() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    // Create an app that uses a string as secret so we can sign our own JWT
    // token.
    let mut settings = Settings::default();
    settings.auth.token_verification = JWTVerification::HS256 {
        secret: "not-a-secret".to_string(),
    };

    let db_dir = tempfile::TempDir::new().unwrap();
    let cs = graphannis::CorpusStorage::with_auto_cache_size(db_dir.path(), false).unwrap();
    let db_pool = create_empty_dbpool();

    let cs = web::Data::new(cs);
    let settings = web::Data::new(settings);
    let db_pool = web::Data::new(db_pool);

    let app = crate::create_app(cs, settings, db_pool);
    app
}

fn create_auth_header() -> (&'static str, String) {
    // Create an auth header for an admin
    let in_sixty_minutes = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .checked_add(Duration::from_secs(3600))
        .unwrap();
    let admin_claims = Claims {
        sub: "admin".to_string(),
        exp: Some(in_sixty_minutes.as_millis() as i64),
        roles: vec!["admin".to_string()],
        groups: vec![],
    };
    let bearer_token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &admin_claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )
    .unwrap();
    ("Authorization", format!("Bearer {bearer_token}"))
}

/// Test several adminstration API end points that they will return an error
/// when no auth info is given.
#[actix_web::test]
async fn needs_bearer_token() {
    let app = test::init_service(create_test_app()).await;

    let req = test::TestRequest::post().uri("/v1/import").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let req = test::TestRequest::post()
        .uri("/v1/export")
        .set_json(ExportParams {
            corpora: vec!["pcc2".to_string()],
        })
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let req = test::TestRequest::get()
        .uri("/v1/jobs/someinvalidid")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let req = test::TestRequest::get().uri("/v1/groups").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let req = test::TestRequest::put()
        .uri("/v1/groups/newgroup")
        .set_json(Group {
            name: "newgroup".to_string(),
            corpora: vec![],
        })
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let req = test::TestRequest::delete()
        .uri("/v1/groups/academic")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn test_user_groups() {
    let app = test::init_service(create_test_app()).await;

    // Initial list of groups, this should be empty
    let req = test::TestRequest::get()
        .insert_header(create_auth_header())
        .uri("/v1/groups")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let response_body: Vec<Group> = test::read_body_json(resp).await;
    assert_eq!(response_body.len(), 0);

    // Add a new group and check it has been persisted
    let req = test::TestRequest::put()
        .insert_header(create_auth_header())
        .uri("/v1/groups/academic")
        .set_json(Group {
            name: "academic".to_string(),
            corpora: vec!["pcc2".to_string(), "GUM".to_string()],
        })
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let req = test::TestRequest::get()
        .insert_header(create_auth_header())
        .uri("/v1/groups")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);
    let response_body: Vec<Group> = test::read_body_json(resp).await;
    assert_eq!(response_body.len(), 1);
    assert_eq!(response_body[0].name, "academic");
    assert_eq!(response_body[0].corpora.len(), 2);
    assert_eq!(response_body[0].corpora[0], "GUM");
    assert_eq!(response_body[0].corpora[1], "pcc2");

    // Delete group again and check it has been removed
    let req = test::TestRequest::delete()
        .insert_header(create_auth_header())
        .uri("/v1/groups/academic")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let req = test::TestRequest::get()
        .insert_header(create_auth_header())
        .uri("/v1/groups")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let response_body: Vec<Group> = test::read_body_json(resp).await;
    assert_eq!(response_body.len(), 0);
}
