//! Tests for package.json utilities

use heisenberg::utils::{infer_from_build_dir, PackageJson};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_infer_from_nonexistent_dir() {
    let result = infer_from_build_dir(std::path::Path::new("/nonexistent/path"));
    assert!(result.is_err());
}

#[test]
fn test_package_json_creation() {
    let mut scripts = std::collections::HashMap::new();
    scripts.insert("dev".to_string(), "vite".to_string());
    scripts.insert("build".to_string(), "vite build".to_string());

    let package_json = PackageJson {
        scripts,
        name: Some("test-app".to_string()),
        version: Some("1.0.0".to_string()),
    };

    assert_eq!(package_json.name, Some("test-app".to_string()));
    assert_eq!(package_json.version, Some("1.0.0".to_string()));
    assert!(package_json.scripts.contains_key("dev"));
}

#[test]
fn test_infer_with_temp_structure() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("project");
    let dist_dir = project_dir.join("dist");

    fs::create_dir_all(&dist_dir).unwrap();

    // Create a basic package.json
    let package_json_content = r#"{
        "name": "test-project",
        "scripts": {
            "dev": "vite --port 3000"
        }
    }"#;

    fs::write(project_dir.join("package.json"), package_json_content).unwrap();

    // Test inference
    let result = infer_from_build_dir(&dist_dir);
    assert!(result.is_ok());

    let config = result.unwrap();
    // Use canonicalized paths for comparison due to macOS symlinks
    assert_eq!(
        config.working_dir.canonicalize().unwrap(),
        project_dir.canonicalize().unwrap()
    );
    assert_eq!(config.dev_port, 3000);
    assert_eq!(config.dev_url, "http://localhost:3000");
}
