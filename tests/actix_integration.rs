//! Actix-web adapter integration tests

#![cfg(feature = "actix")]

use actix_web::test;
use heisenberg::{adapters::actix::serve_spa, EmbeddedSpa, Heisenberg};

fn register_test_assets() {
    use std::fs;
    let base_path = "./tests/fixtures/minimal-spa/dist";
    heisenberg::services::embed_registry::register_embedded_assets(base_path, move |path| {
        let full_path = format!("{}/{}", base_path, path);
        fs::read(&full_path).ok()
    });
}

#[actix_web::test]
async fn test_actix_serve_spa_basic() {
    std::env::set_var("HEISENBERG_MODE", "embed");
    register_test_assets();

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();
    let req = test::TestRequest::get().uri("/").to_http_request();

    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());
}

#[actix_web::test]
async fn test_actix_path_matching() {
    std::env::set_var("HEISENBERG_MODE", "embed");
    register_test_assets();

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();

    let req = test::TestRequest::get().uri("/").to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());

    let req = test::TestRequest::get().uri("/app/home").to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());
}
