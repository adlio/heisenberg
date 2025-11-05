//! Embedded SPA handle

use std::path::PathBuf;

/// Handle to an embedded SPA's assets and configuration
///
/// Created by the `embed_spa!` macro. Pass this to `Heisenberg::spa()` to configure routing.
#[derive(Debug, Clone)]
pub struct EmbeddedSpa {
    /// Path to the SPA directory (contains package.json and build output)
    pub(crate) spa_dir: PathBuf,
    /// Subdirectory containing built assets (e.g., "dist", "build")
    pub(crate) build_subdir: String,
}

impl EmbeddedSpa {
    /// Create a new embedded SPA handle
    #[doc(hidden)]
    pub fn new(spa_dir: impl Into<PathBuf>, build_subdir: impl Into<String>) -> Self {
        Self {
            spa_dir: spa_dir.into(),
            build_subdir: build_subdir.into(),
        }
    }

    /// Get the full path to the build directory
    pub(crate) fn build_path(&self) -> PathBuf {
        if self.build_subdir.is_empty() {
            self.spa_dir.clone()
        } else {
            self.spa_dir.join(&self.build_subdir)
        }
    }
}
