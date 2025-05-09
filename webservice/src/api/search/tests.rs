use actix_web::http::StatusCode;
use graphannis::corpusstorage::CountExtra;

use crate::tests::{create_auth_header, create_test_app, import_test_corpora};

use super::*;

#[actix_web::test]
async fn test_count() {
    let db_dir = tempfile::TempDir::new().unwrap();
    let cs = graphannis::CorpusStorage::with_auto_cache_size(db_dir.path(), false).unwrap(); // Import three corpora A,B and C
    import_test_corpora(&cs);

    let app =
        actix_web::test::init_service(create_test_app(web::Data::new(cs), Settings::default()))
            .await;

    // Execute a query to load a corpus
    let req = actix_web::test::TestRequest::post()
        .uri("/v1/search/count")
        .set_json(CountQuery {
            query: "tok".into(),
            query_language: QueryLanguage::AQL,
            corpora: vec!["A".into()],
        })
        .insert_header(create_auth_header())
        .to_request();
    let resp = actix_web::test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let response_body: CountExtra = actix_web::test::read_body_json(resp).await;

    assert_eq!(response_body.document_count, 4);
    assert_eq!(response_body.match_count, 44);
}
