//! End-to-end HTTP serving tests

mod common;

use axum::{routing::get, Router};
use common::EnvGuard;
use heisenberg::{EmbeddedSpa, Heisenberg, HeisenbergLayer};
use serial_test::serial;
use tokio::net::TcpListener;

#[tokio::test]
#[serial]
async fn test_config_with_embedded_spa() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();

    assert_eq!(config.routes().len(), 1);
    assert_eq!(config.routes()[0].pattern, "/*");
    assert!(config.routes()[0].embed_dir.ends_with("dist"));
}

#[tokio::test]
async fn test_route_pattern_validation() {
    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/app/*", spa).build();

    assert_eq!(config.routes().len(), 1);
    assert_eq!(config.routes()[0].pattern, "/app/*");
}

#[tokio::test]
#[serial]
async fn test_api_routes_not_intercepted() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();

    let app = Router::new()
        .route("/api/test", get(|| async { "API response" }))
        .layer(HeisenbergLayer::new(config));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{}/api/test", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    assert_eq!(body, "API response");
}

#[tokio::test]
async fn test_multiple_spa_configuration() {
    let spa1 = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "admin");
    let spa2 = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "app");
    let config = Heisenberg::new()
        .route("/admin/*", spa1)
        .route("/*", spa2)
        .build();

    assert_eq!(config.routes().len(), 2);
    assert_eq!(config.routes()[0].pattern, "/admin/*");
    assert_eq!(config.routes()[1].pattern, "/*");
}
