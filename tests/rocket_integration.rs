//! Rocket adapter integration tests

#![cfg(feature = "rocket")]

use heisenberg::{adapters::rocket::serve_spa, EmbeddedSpa, Heisenberg};

#[tokio::test]
async fn test_rocket_serve_spa_basic() {
    std::env::set_var("HEISENBERG_MODE", "embed");

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();

    let result = serve_spa("index.html", &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rocket_path_matching() {
    std::env::set_var("HEISENBERG_MODE", "embed");

    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();

    let result = serve_spa("index.html", &config).await;
    assert!(result.is_ok());

    let result = serve_spa("app/home", &config).await;
    assert!(result.is_ok());
}
