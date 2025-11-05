//! Actix-web adapter integration tests

#![cfg(feature = "actix")]

use actix_web::test;
use heisenberg::{adapters::actix::serve_spa, EmbeddedSpa, Heisenberg};

#[actix_web::test]
async fn test_actix_serve_spa_basic() {
    std::env::set_var("HEISENBERG_MODE", "embed");

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();
    let req = test::TestRequest::get().uri("/").to_http_request();

    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());
}

#[actix_web::test]
async fn test_actix_path_matching() {
    std::env::set_var("HEISENBERG_MODE", "embed");

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();

    let req = test::TestRequest::get().uri("/").to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());

    let req = test::TestRequest::get().uri("/app/home").to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());
}
