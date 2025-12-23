//! Rocket adapter integration tests

#![cfg(feature = "rocket")]

use heisenberg::{adapters::rocket::serve_spa, EmbeddedSpa, Heisenberg};

fn register_test_assets() {
    use std::fs;
    let base_path = "./tests/fixtures/minimal-spa/dist";
    heisenberg::services::embed_registry::register_embedded_assets(base_path, move |path| {
        let full_path = format!("{}/{}", base_path, path);
        fs::read(&full_path).ok()
    });
}

#[tokio::test]
async fn test_rocket_serve_spa_basic() {
    register_test_assets();

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();

    let result = serve_spa("index.html", &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rocket_path_matching() {
    register_test_assets();

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();

    let result = serve_spa("index.html", &config).await;
    assert!(result.is_ok());

    let result = serve_spa("app/home", &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rocket_prefix_pattern_matching() {
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
async fn test_rocket_exact_pattern_matching() {
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
async fn test_rocket_multiple_routes() {
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
async fn test_rocket_query_string_preserved() {
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
