//! Tower integration tests

#![cfg(feature = "axum")]

mod common;

use common::EnvGuard;
use heisenberg::{EmbeddedSpa, Heisenberg, HeisenbergLayer};
use serial_test::serial;
use tower::ServiceBuilder;

#[test]
#[serial]
fn test_tower_layer_creation() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();
    let _layer = HeisenbergLayer::new(config);
}

#[test]
#[serial]
fn test_tower_layer_with_multiple_routes() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
    let spa1 = EmbeddedSpa::new("./admin-dist", "");
    let spa2 = EmbeddedSpa::new("./app-dist", "");
    let config = Heisenberg::new()
        .route("/admin/*", spa1)
        .route("/*", spa2)
        .build();
    let _layer = HeisenbergLayer::new(config);
}

#[test]
#[serial]
fn test_service_builder_integration() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();
    let layer = HeisenbergLayer::new(config);

    let _service_builder = ServiceBuilder::new().layer(layer);
}

#[test]
#[serial]
fn test_layer_debug_format() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");
    let spa = EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new().route("/*", spa).build();
    let layer = HeisenbergLayer::new(config);

    let debug_str = format!("{:?}", layer);
    assert!(debug_str.contains("HeisenbergLayer"));
}
