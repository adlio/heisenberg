//! Tower layer implementation

use crate::core::config::Heisenberg;
use crate::tower::service::HeisenbergService;
use tower_layer::Layer;

/// Tower layer for Heisenberg dual-mode serving
#[derive(Debug, Clone)]
pub struct HeisenbergLayer {
    config: Heisenberg,
}

impl HeisenbergLayer {
    /// Create a new Heisenberg layer
    pub fn new(config: Heisenberg) -> Self {
        Self { config }
    }
}

impl<S> Layer<S> for HeisenbergLayer {
    type Service = HeisenbergService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HeisenbergService::new(inner, self.config.clone())
            .expect("Failed to create HeisenbergService with router")
    }
}
