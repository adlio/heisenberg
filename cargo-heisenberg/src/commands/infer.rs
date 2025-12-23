//! Shared SPA configuration inference logic.

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// Find the frontend working directory (web or frontend).
pub fn find_frontend_dir(base_dir: &Path) -> Result<PathBuf> {
    if base_dir.join("web/package.json").exists() {
        Ok(PathBuf::from("./web"))
    } else if base_dir.join("frontend/package.json").exists() {
        Ok(PathBuf::from("./frontend"))
    } else {
        bail!(
            "No frontend found. Create heisenberg.toml or add ./web/package.json or ./frontend/package.json"
        )
    }
}

/// Find the output directory based on common framework patterns.
pub fn find_output_dir(working_dir: &Path) -> PathBuf {
    let output_subdir = ["build", "dist", ".next", ".svelte-kit/output"]
        .iter()
        .find(|d| working_dir.join(d).exists())
        .unwrap_or(&"build");
    working_dir.join(output_subdir)
}

/// Infer build command from package.json scripts.
pub fn infer_build_command(working_dir: &Path) -> Option<String> {
    let content = std::fs::read_to_string(working_dir.join("package.json")).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    json.get("scripts")
        .and_then(|s| s.as_object())
        .and_then(|scripts| {
            ["build", "build:release", "build:dist"]
                .iter()
                .find(|name| scripts.contains_key(**name))
                .map(|name| format!("npm run {}", name))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_frontend_dir_web() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("web")).unwrap();
        fs::write(temp.path().join("web/package.json"), "{}").unwrap();

        assert_eq!(
            find_frontend_dir(temp.path()).unwrap(),
            PathBuf::from("./web")
        );
    }

    #[test]
    fn test_find_frontend_dir_frontend() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("frontend")).unwrap();
        fs::write(temp.path().join("frontend/package.json"), "{}").unwrap();

        assert_eq!(
            find_frontend_dir(temp.path()).unwrap(),
            PathBuf::from("./frontend")
        );
    }

    #[test]
    fn test_find_frontend_dir_prefers_web() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("web")).unwrap();
        fs::create_dir_all(temp.path().join("frontend")).unwrap();
        fs::write(temp.path().join("web/package.json"), "{}").unwrap();
        fs::write(temp.path().join("frontend/package.json"), "{}").unwrap();

        assert_eq!(
            find_frontend_dir(temp.path()).unwrap(),
            PathBuf::from("./web")
        );
    }

    #[test]
    fn test_find_frontend_dir_not_found() {
        let temp = TempDir::new().unwrap();
        assert!(find_frontend_dir(temp.path()).is_err());
    }

    #[test]
    fn test_find_output_dir_existing() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("dist")).unwrap();

        assert_eq!(find_output_dir(temp.path()), temp.path().join("dist"));
    }

    #[test]
    fn test_find_output_dir_default() {
        let temp = TempDir::new().unwrap();
        assert_eq!(find_output_dir(temp.path()), temp.path().join("build"));
    }

    #[test]
    fn test_infer_build_command_found() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("package.json"),
            r#"{"scripts":{"build":"vite build"}}"#,
        )
        .unwrap();

        assert_eq!(
            infer_build_command(temp.path()),
            Some("npm run build".to_string())
        );
    }

    #[test]
    fn test_infer_build_command_not_found() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("package.json"),
            r#"{"scripts":{"dev":"vite"}}"#,
        )
        .unwrap();

        assert_eq!(infer_build_command(temp.path()), None);
    }
}
