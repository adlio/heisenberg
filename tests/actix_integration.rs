//! Actix-web adapter integration tests

#![cfg(feature = "actix")]

use actix_web::test;
use heisenberg::{adapters::actix::serve_spa, Heisenberg};

#[actix_web::test]
async fn test_actix_serve_spa_basic() {
    // Force production mode for testing
    std::env::set_var("HEISENBERG_MODE", "production");

    let config = Heisenberg::new().spa("./test-dist").build();
    let req = test::TestRequest::get().uri("/").to_http_request();

    // Test that the function can be called without panicking
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());
}

#[actix_web::test]
async fn test_actix_path_matching() {
    // Force production mode for testing
    std::env::set_var("HEISENBERG_MODE", "production");

    let config = Heisenberg::new().spa("./test-dist").build();

    // Test basic route
    let req = test::TestRequest::get().uri("/").to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());

    // Test nested path
    let req = test::TestRequest::get().uri("/app/home").to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());
}
