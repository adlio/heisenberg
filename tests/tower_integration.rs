//! Tower integration tests

#![cfg(feature = "axum")]

use heisenberg::{EmbeddedSpa, Heisenberg, HeisenbergLayer};
use tower::ServiceBuilder;

#[test]
fn test_tower_layer_creation() {
    std::env::set_var("HEISENBERG_MODE", "embed");
    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();
    let _layer = HeisenbergLayer::new(config);
    std::env::remove_var("HEISENBERG_MODE");
}

#[test]
fn test_tower_layer_with_multiple_routes() {
    std::env::set_var("HEISENBERG_MODE", "embed");
    let spa1 = EmbeddedSpa::new("./admin-dist", "");
    let spa2 = EmbeddedSpa::new("./app-dist", "");
    let config = Heisenberg::new()
        .route("/admin/*", spa1)
        .route("/*", spa2)
        .build();
    let _layer = HeisenbergLayer::new(config);
    std::env::remove_var("HEISENBERG_MODE");
}

#[test]
fn test_service_builder_integration() {
    std::env::set_var("HEISENBERG_MODE", "embed");
    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();
    let layer = HeisenbergLayer::new(config);

    let _service_builder = ServiceBuilder::new().layer(layer);
    std::env::remove_var("HEISENBERG_MODE");
}

#[test]
fn test_layer_debug_format() {
    std::env::set_var("HEISENBERG_MODE", "embed");
    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();
    let layer = HeisenbergLayer::new(config);

    let debug_str = format!("{:?}", layer);
    assert!(debug_str.contains("HeisenbergLayer"));
    std::env::remove_var("HEISENBERG_MODE");
}
