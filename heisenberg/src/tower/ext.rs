//! Extension traits for ergonomic integration

use crate::core::config::Heisenberg;
use crate::tower::HeisenbergLayer;
use crate::EmbeddedSpa;

/// Extension trait for Router-like types to add SPA support
pub trait SpaExt: Sized {
    /// Add SPA with automatic detection
    ///
    /// Looks for ./web, ./frontend, or reads heisenberg.toml
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let app = Router::new()
    ///     .route("/api/hello", get(handler))
    ///     .spa();
    /// ```
    fn spa(self) -> Self {
        self.spa_from("./web")
    }

    /// Add SPA from working directory (where package.json lives)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let app = Router::new()
    ///     .route("/api/hello", get(handler))
    ///     .spa_from("./web");  // References the app, not the output
    /// ```
    fn spa_from(self, working_dir: &str) -> Self;

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
    fn spa_from(self, working_dir: &str) -> Self {
        self.spa_with_route("/*", working_dir)
    }

    fn spa_with_route(self, route: &str, working_dir: &str) -> Self {
        // Infer output directory from working directory
        let output_dir = infer_output_dir(working_dir);
        let embedded = EmbeddedSpa::new(&output_dir, "");
        let config = Heisenberg::new().route(route, embedded).build();
        self.layer(HeisenbergLayer::new(config))
    }
}

fn infer_output_dir(working_dir: &str) -> String {
    use std::path::PathBuf;

    // Try common output directories
    for candidate in &["build", "dist", ".next", ".svelte-kit/output"] {
        let path = PathBuf::from(working_dir).join(candidate);
        if path.exists() {
            return path.to_string_lossy().to_string();
        }
    }

    // Default to build
    PathBuf::from(working_dir)
        .join("build")
        .to_string_lossy()
        .to_string()
}
