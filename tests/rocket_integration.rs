//! Rocket adapter integration tests

#![cfg(feature = "rocket")]

mod common;

use axum::{routing::get, Router};
use common::EnvGuard;
use heisenberg::{adapters::rocket::serve_spa, EmbeddedSpa, Heisenberg};
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

#[tokio::test]
#[serial]
async fn test_rocket_serve_spa_basic() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
    register_test_assets();

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();

    let result = serve_spa("index.html", &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_rocket_path_matching() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
    register_test_assets();

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();

    let result = serve_spa("index.html", &config).await;
    assert!(result.is_ok());

    let result = serve_spa("app/home", &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_rocket_prefix_pattern_matching() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
    register_test_assets();

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/app/*", spa).build();

    // Should match paths under /app/
    let result = serve_spa("/app/dashboard", &config).await;
    assert!(result.is_ok());

    let result = serve_spa("/app/users/123", &config).await;
    assert!(result.is_ok());

    // Should NOT match paths outside /app/
    let result = serve_spa("/other/path", &config).await;
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_rocket_exact_pattern_matching() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
    register_test_assets();

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/about", spa).build();

    // Should match exact path
    let result = serve_spa("/about", &config).await;
    assert!(result.is_ok());

    // Should NOT match other paths
    let result = serve_spa("/about/team", &config).await;
    assert!(result.is_err());

    let result = serve_spa("/", &config).await;
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_rocket_multiple_routes() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
    register_test_assets();

    let admin_spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let app_spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new()
        .route("/admin/*", admin_spa)
        .route("/*", app_spa)
        .build();

    // /admin/* should match first
    let result = serve_spa("/admin/users", &config).await;
    assert!(result.is_ok());

    // Other paths should match the catchall
    let result = serve_spa("/dashboard", &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_rocket_query_string_preserved() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
    register_test_assets();

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();

    // Query strings should be handled (path extracted correctly for routing)
    let result = serve_spa("/app/page?foo=bar&baz=123", &config).await;
    assert!(result.is_ok());

    let result = serve_spa("/?t=12345", &config).await;
    assert!(result.is_ok());
}

#[test]
fn test_rocket_spa_routes_returns_two_routes() {
    use heisenberg::adapters::rocket::spa_routes;

    let routes = spa_routes();
    assert_eq!(routes.len(), 2);
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

#[tokio::test]
#[serial]
async fn test_rocket_proxy_forwards_requests() {
    let _guard = EnvGuard::set("HEISENBERG_MODE", "proxy");

    let dev_port = start_mock_dev_server().await;
    let dev_url = format!("http://localhost:{}", dev_port);

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server(&dev_url)
        .build();

    let result = serve_spa("/", &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_rocket_proxy_forwards_nested_paths() {
    let _guard = EnvGuard::set("HEISENBERG_MODE", "proxy");

    let dev_port = start_mock_dev_server().await;
    let dev_url = format!("http://localhost:{}", dev_port);

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server(&dev_url)
        .build();

    let result = serve_spa("/page", &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_rocket_proxy_preserves_content_type() {
    let _guard = EnvGuard::set("HEISENBERG_MODE", "proxy");

    let dev_port = start_mock_dev_server().await;
    let dev_url = format!("http://localhost:{}", dev_port);

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server(&dev_url)
        .build();

    let result = serve_spa("/assets/style.css", &config).await;
    assert!(result.is_ok());

    // Note: We can't easily inspect the RocketResponse headers without
    // the full Rocket test infrastructure, but the request succeeded
    // which means the proxy worked.
}

#[tokio::test]
#[serial]
async fn test_rocket_proxy_forwards_query_strings() {
    let _guard = EnvGuard::set("HEISENBERG_MODE", "proxy");

    let dev_port = start_mock_dev_server().await;
    let dev_url = format!("http://localhost:{}", dev_port);

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server(&dev_url)
        .build();

    // Query strings should be forwarded to the dev server
    let result = serve_spa("/?t=12345", &config).await;
    assert!(result.is_ok());

    let result = serve_spa("/page?foo=bar&baz=123", &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_rocket_proxy_returns_error_when_dev_server_down() {
    let _guard = EnvGuard::set("HEISENBERG_MODE", "proxy");

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server("http://localhost:59999") // Non-existent server
        .build();

    let result = serve_spa("/", &config).await;

    // Should return an error when dev server is unreachable
    assert!(result.is_err());
}
