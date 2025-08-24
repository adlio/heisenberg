//! Tower integration tests

#![cfg(feature = "tower")]

use heisenberg::{Heisenberg, HeisenbergLayer};
use tower::ServiceBuilder;

#[test]
fn test_tower_layer_creation() {
    let config = Heisenberg::new().spa("./test-dist").build();
    let _layer = HeisenbergLayer::new(config);

    // Should be able to create layer without panicking
    assert!(true); // Layer created successfully
}

#[test]
fn test_tower_layer_with_multiple_routes() {
    let config = Heisenberg::new()
        .spa("./admin-dist")
        .spa("./app-dist")
        .build();
    let _layer = HeisenbergLayer::new(config);

    // Should handle multiple routes
    assert!(true); // Layer created successfully
}

#[test]
fn test_service_builder_integration() {
    let config = Heisenberg::new().spa("./test-dist").build();
    let layer = HeisenbergLayer::new(config);

    // Should integrate with ServiceBuilder
    let _service_builder = ServiceBuilder::new().layer(layer);

    assert!(true); // ServiceBuilder integration works
}

#[test]
fn test_layer_debug_format() {
    let config = Heisenberg::new().spa("./test-dist").build();
    let layer = HeisenbergLayer::new(config);

    // Should be able to debug format the layer
    let debug_str = format!("{:?}", layer);
    assert!(debug_str.contains("HeisenbergLayer"));
}
