//! Tower integration tests

#![cfg(feature = "tower")]

use heisenberg::{Heisenberg, HeisenbergLayer};
use tower::ServiceBuilder;

#[test]
fn test_tower_layer_creation() {
    let config = Heisenberg::new().spa("./test-dist").build();
    let _layer = HeisenbergLayer::new(config);
    // Layer created successfully without panicking
}

#[test]
fn test_tower_layer_with_multiple_routes() {
    let config = Heisenberg::new()
        .spa("./admin-dist")
        .spa("./app-dist")
        .build();
    let _layer = HeisenbergLayer::new(config);
    // Layer handles multiple routes successfully
}

#[test]
fn test_service_builder_integration() {
    let config = Heisenberg::new().spa("./test-dist").build();
    let layer = HeisenbergLayer::new(config);

    let _service_builder = ServiceBuilder::new().layer(layer);
    // ServiceBuilder integration works successfully
}

#[test]
fn test_layer_debug_format() {
    let config = Heisenberg::new().spa("./test-dist").build();
    let layer = HeisenbergLayer::new(config);

    // Should be able to debug format the layer
    let debug_str = format!("{:?}", layer);
    assert!(debug_str.contains("HeisenbergLayer"));
}
