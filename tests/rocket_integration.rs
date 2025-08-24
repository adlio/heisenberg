//! Rocket adapter integration tests

#![cfg(feature = "rocket")]

use heisenberg::{adapters::rocket::serve_spa, Heisenberg};
use std::path::PathBuf;

#[tokio::test]
async fn test_rocket_serve_spa_basic() {
    // Force production mode for testing
    std::env::set_var("HEISENBERG_MODE", "production");

    let config = Heisenberg::new().spa("./test-dist").build();
    let path = PathBuf::from("index.html");

    // Test that the function can be called without panicking
    let result = serve_spa(&path, &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rocket_path_matching() {
    // Force production mode for testing
    std::env::set_var("HEISENBERG_MODE", "production");

    let config = Heisenberg::new().spa("./test-dist").build();

    // Test basic route
    let path = PathBuf::from("index.html");
    let result = serve_spa(&path, &config).await;
    assert!(result.is_ok());

    // Test nested path
    let path = PathBuf::from("app/home");
    let result = serve_spa(&path, &config).await;
    assert!(result.is_ok());
}
