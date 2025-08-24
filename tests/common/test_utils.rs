//! Shared testing utilities for Heisenberg integration tests

use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixture for creating temporary SPA projects
pub struct TestSpaFixture {
    #[allow(dead_code)] // Will be used in Phase 3 for process management
    pub temp_dir: TempDir,
    pub dist_dir: PathBuf,
    #[allow(dead_code)] // Will be used in Phase 3 for package.json inference
    pub package_json_path: PathBuf,
}

impl TestSpaFixture {
    /// Create a new test SPA fixture with basic structure
    pub fn new() -> std::io::Result<Self> {
        let temp_dir = TempDir::new()?;
        let dist_dir = temp_dir.path().join("dist");
        let package_json_path = temp_dir.path().join("package.json");

        // Create dist directory
        std::fs::create_dir_all(&dist_dir)?;

        // Create basic package.json
        let package_json = r#"{
  "name": "test-spa",
  "version": "1.0.0",
  "scripts": {
    "dev": "vite",
    "build": "vite build"
  }
}"#;
        std::fs::write(&package_json_path, package_json)?;

        // Create basic index.html in dist
        let index_html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test SPA</title>
</head>
<body>
    <div id="app">Test SPA Content</div>
</body>
</html>"#;
        std::fs::write(dist_dir.join("index.html"), index_html)?;

        Ok(Self {
            temp_dir,
            dist_dir,
            package_json_path,
        })
    }

    /// Get the path to the dist directory
    pub fn dist_path(&self) -> &PathBuf {
        &self.dist_dir
    }

    /// Get the working directory path
    #[allow(dead_code)] // Will be used in Phase 3 for process management
    pub fn working_dir(&self) -> &std::path::Path {
        self.temp_dir.path()
    }
}
