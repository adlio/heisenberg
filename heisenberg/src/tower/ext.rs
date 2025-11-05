//! Extension traits for ergonomic integration

use crate::core::config::Heisenberg;
use crate::tower::HeisenbergLayer;
use crate::EmbeddedSpa;

/// Extension trait for Router-like types to add SPA support
pub trait SpaExt: Sized {
    /// Add SPA with automatic detection at root (/*)
    ///
    /// Looks for ./web or ./frontend
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let app = Router::new()
    ///     .route("/api/hello", get(handler))
    ///     .spa();  // Mounts at /*
    /// ```
    fn spa(self) -> Self {
        self.spa_at("/*")
    }

    /// Add SPA at specific route pattern
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Single SPA at root
    /// let app = Router::new()
    ///     .route("/api/hello", get(handler))
    ///     .spa_at("/*");
    ///
    /// // Multiple SPAs
    /// let app = Router::new()
    ///     .route("/api/hello", get(handler))
    ///     .spa_at("/admin/*")
    ///     .spa_at("/app/*");
    /// ```
    fn spa_at(self, route: &str) -> Self {
        // Infer working dir from route
        let working_dir = if route.starts_with("/admin") {
            "./admin"
        } else if route.starts_with("/app") {
            "./app"
        } else {
            "./web"
        };
        self.spa_at_from(route, working_dir)
    }

    /// Add SPA at route from specific working directory
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let app = Router::new()
    ///     .route("/api/hello", get(handler))
    ///     .spa_at_from("/admin/*", "./admin")
    ///     .spa_at_from("/app/*", "./frontend");
    /// ```
    fn spa_at_from(self, route: &str, working_dir: &str) -> Self;
}

impl<S> SpaExt for axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn spa_at_from(self, route: &str, working_dir: &str) -> Self {
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
