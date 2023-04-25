use std::path::PathBuf;

use actix_web::{http::StatusCode, test, web};
use graphannis::{corpusstorage::ImportFormat, CorpusStorage};

use crate::{
    api::administration::Group,
    tests::{create_auth_header, create_test_app},
};
use pretty_assertions::assert_eq;

fn import_test_corpora(cs: &CorpusStorage) {
    let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Import three corpora A,B and C
    cs.import_from_fs(
        &cargo_dir.join("../graphannis/tests/SaltSampleCorpus.graphml"),
        ImportFormat::GraphML,
        Some("A".into()),
        false,
        true,
        |_| {},
    )
    .unwrap();

    cs.import_from_fs(
        &cargo_dir.join("../graphannis/tests/SaltSampleCorpus.graphml"),
        ImportFormat::GraphML,
        Some("B".into()),
        false,
        true,
        |_| {},
    )
    .unwrap();

    cs.import_from_fs(
        &cargo_dir.join("../graphannis/tests/SaltSampleCorpus.graphml"),
        ImportFormat::GraphML,
        Some("C".into()),
        false,
        true,
        |_| {},
    )
    .unwrap();
}

#[actix_web::test]
async fn list_corpora() {
    let db_dir = tempfile::TempDir::new().unwrap();
    let cs = graphannis::CorpusStorage::with_auto_cache_size(db_dir.path(), false).unwrap(); // Import three corpora A,B and C
    import_test_corpora(&cs);

    let app = test::init_service(create_test_app(web::Data::new(cs))).await;

    // Unauthorized user should not see any corpora
    let req = test::TestRequest::get().uri("/v1/corpora").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let response_body: Vec<String> = test::read_body_json(resp).await;
    assert_eq!(response_body.len(), 0);

    // Admin user should see all corpora
    let req = test::TestRequest::get()
        .insert_header(create_auth_header())
        .uri("/v1/corpora")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let response_body: Vec<String> = test::read_body_json(resp).await;
    assert_eq!(response_body.len(), 3);
    assert_eq!(response_body[0], "A");
    assert_eq!(response_body[1], "B");
    assert_eq!(response_body[2], "C");

    // Add a group configuration for anonymous users and repeat corpus list call
    let req = test::TestRequest::put()
        .insert_header(create_auth_header())
        .uri("/v1/groups/anonymous")
        .set_json(Group {
            name: "anonymous".to_string(),
            corpora: vec!["B".to_string()],
        })
        .to_request();
    test::call_service(&app, req).await;
    let req = test::TestRequest::get().uri("/v1/corpora").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let response_body: Vec<String> = test::read_body_json(resp).await;
    assert_eq!(response_body.len(), 1);
    assert_eq!(response_body[0], "B");
}
