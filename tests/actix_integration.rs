//! Actix-web adapter integration tests

#![cfg(feature = "actix")]

mod common;

use actix_web::test;
use axum::{routing::get, Router};
use common::EnvGuard;
use heisenberg::{adapters::actix::serve_spa, EmbeddedSpa, Heisenberg};
use serial_test::serial;
use tokio::net::TcpListener;

fn register_test_assets() {
    use std::fs;
    let base_path = "./tests/fixtures/minimal-spa/dist";
    heisenberg::services::embed_registry::register_embedded_assets(base_path, move |path| {
        let full_path = format!("{}/{}", base_path, path);
        fs::read(&full_path).ok()
    });
}

#[actix_web::test]
#[serial]
async fn test_actix_serve_spa_basic() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
    register_test_assets();

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();
    let req = test::TestRequest::get().uri("/").to_http_request();

    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());
}

#[actix_web::test]
#[serial]
async fn test_actix_path_matching() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
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
#[serial]
async fn test_actix_prefix_pattern_matching() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
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
#[serial]
async fn test_actix_exact_pattern_matching() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
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
#[serial]
async fn test_actix_multiple_routes() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
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

// =============================================================================
// Proxy Mode Tests
// =============================================================================

async fn start_mock_dev_server() -> u16 {
    let app = Router::new()
        .route("/", get(|| async { "<html>Dev Server Root</html>" }))
        .route("/page", get(|| async { "<html>Dev Server Page</html>" }))
        .route(
            "/assets/style.css",
            get(|| async { ([("content-type", "text/css")], "body { color: red; }") }),
        );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    port
}

#[actix_web::test]
#[serial]
async fn test_actix_proxy_forwards_requests() {
    let _guard = EnvGuard::set("HEISENBERG_MODE", "proxy");

    let dev_port = start_mock_dev_server().await;
    let dev_url = format!("http://localhost:{}", dev_port);

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server(&dev_url)
        .build();

    let req = test::TestRequest::get().uri("/").to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
#[serial]
async fn test_actix_proxy_forwards_nested_paths() {
    let _guard = EnvGuard::set("HEISENBERG_MODE", "proxy");

    let dev_port = start_mock_dev_server().await;
    let dev_url = format!("http://localhost:{}", dev_port);

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server(&dev_url)
        .build();

    let req = test::TestRequest::get().uri("/page").to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
#[serial]
async fn test_actix_proxy_preserves_content_type() {
    let _guard = EnvGuard::set("HEISENBERG_MODE", "proxy");

    let dev_port = start_mock_dev_server().await;
    let dev_url = format!("http://localhost:{}", dev_port);

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server(&dev_url)
        .build();

    let req = test::TestRequest::get()
        .uri("/assets/style.css")
        .to_http_request();
    let result = serve_spa(&req, &config).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.status(), actix_web::http::StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok());
    assert!(content_type.is_some());
    assert!(content_type.unwrap().contains("text/css"));
}

#[actix_web::test]
#[serial]
async fn test_actix_proxy_returns_error_when_dev_server_down() {
    let _guard = EnvGuard::set("HEISENBERG_MODE", "proxy");

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server("http://localhost:59999") // Non-existent server
        .build();

    let req = test::TestRequest::get().uri("/").to_http_request();
    let result = serve_spa(&req, &config).await;

    // Should return an error when dev server is unreachable
    assert!(result.is_err());
}
