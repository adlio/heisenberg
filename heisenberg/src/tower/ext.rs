//! Extension traits for ergonomic integration

use crate::core::config::Heisenberg;
use crate::tower::HeisenbergLayer;
use crate::EmbeddedSpa;

/// Extension trait for Router-like types to add SPA support
pub trait SpaExt: Sized {
    /// Add SPA with automatic path detection
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let app = Router::new()
    ///     .route("/api/hello", get(handler))
    ///     .spa_auto();
    /// ```
    fn spa_auto(self) -> Self {
        self.spa("./web/build")
    }

    /// Add SPA with explicit path
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let app = Router::new()
    ///     .route("/api/hello", get(handler))
    ///     .spa("./dist");
    /// ```
    fn spa(self, path: &str) -> Self;

    /// Add SPA with custom route pattern
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let app = Router::new()
    ///     .route("/api/hello", get(handler))
    ///     .spa_with_route("/app/*", "./dist");
    /// ```
    fn spa_with_route(self, route: &str, path: &str) -> Self;
}

impl<S> SpaExt for axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn spa(self, path: &str) -> Self {
        self.spa_with_route("/*", path)
    }

    fn spa_with_route(self, route: &str, path: &str) -> Self {
        let embedded = EmbeddedSpa::new(path, "");
        let config = Heisenberg::new().route(route, embedded).build();
        self.layer(HeisenbergLayer::new(config))
    }
}
