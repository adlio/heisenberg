//! Tower integration tests

#![cfg(feature = "tower")]

use heisenberg::{Heisenberg, HeisenbergLayer};
use tower::ServiceBuilder;

#[test]
fn test_tower_layer_creation() {
    std::env::set_var("HEISENBERG_MODE", "embed");
    let config = Heisenberg::new().spa("./test-dist").build();
    let _layer = HeisenbergLayer::new(config);
    std::env::remove_var("HEISENBERG_MODE");
    // Layer created successfully without panicking
}

#[test]
fn test_tower_layer_with_multiple_routes() {
    std::env::set_var("HEISENBERG_MODE", "embed");
    let config = Heisenberg::new()
        .spa("./admin-dist")
        .spa("./app-dist")
        .build();
    let _layer = HeisenbergLayer::new(config);
    std::env::remove_var("HEISENBERG_MODE");
    // Layer handles multiple routes successfully
}

#[test]
fn test_service_builder_integration() {
    std::env::set_var("HEISENBERG_MODE", "embed");
    let config = Heisenberg::new().spa("./test-dist").build();
    let layer = HeisenbergLayer::new(config);

    let _service_builder = ServiceBuilder::new().layer(layer);
    std::env::remove_var("HEISENBERG_MODE");
    // ServiceBuilder integration works successfully
}

#[test]
fn test_layer_debug_format() {
    std::env::set_var("HEISENBERG_MODE", "embed");
    let config = Heisenberg::new().spa("./test-dist").build();
    let layer = HeisenbergLayer::new(config);

    // Should be able to debug format the layer
    let debug_str = format!("{:?}", layer);
    assert!(debug_str.contains("HeisenbergLayer"));
    std::env::remove_var("HEISENBERG_MODE");
}
