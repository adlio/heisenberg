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
    register_test_assets();

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();
    let req = test::TestRequest::get().uri("/").to_http_request();

    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());
}

#[actix_web::test]
async fn test_actix_path_matching() {
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

#[actix_web::test]
async fn test_actix_prefix_pattern_matching() {
    register_test_assets();

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/app/*", spa).build();

    // Should match paths under /app/
    let req = test::TestRequest::get()
        .uri("/app/dashboard")
        .to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());

    let req = test::TestRequest::get()
        .uri("/app/users/123")
        .to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());

    // Should NOT match paths outside /app/
    let req = test::TestRequest::get()
        .uri("/other/path")
        .to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_err());
}

#[actix_web::test]
async fn test_actix_exact_pattern_matching() {
    register_test_assets();

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/about", spa).build();

    // Should match exact path
    let req = test::TestRequest::get().uri("/about").to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());

    // Should NOT match other paths
    let req = test::TestRequest::get()
        .uri("/about/team")
        .to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_err());

    let req = test::TestRequest::get().uri("/").to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_err());
}

#[actix_web::test]
async fn test_actix_multiple_routes() {
    register_test_assets();

    let admin_spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let app_spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new()
        .route("/admin/*", admin_spa)
        .route("/*", app_spa)
        .build();

    // /admin/* should match first
    let req = test::TestRequest::get()
        .uri("/admin/users")
        .to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());

    // Other paths should match the catchall
    let req = test::TestRequest::get().uri("/dashboard").to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());
}
