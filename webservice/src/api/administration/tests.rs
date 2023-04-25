use crate::tests::{create_auth_header, create_test_app};

use super::*;
use actix_web::{
    http::{self, StatusCode},
    test,
};
use pretty_assertions::assert_eq;

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
