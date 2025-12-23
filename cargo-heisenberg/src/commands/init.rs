use anyhow::{Context, Result};
use heisenberg::config::{HeisenbergConfig, SpaConfig};
use std::fs;
use std::path::{Path, PathBuf};

/// Run the init command in the current directory.
pub fn run() -> Result<()> {
    run_in_dir(Path::new("."))
}

/// Run the init command in the specified directory.
pub fn run_in_dir(base_dir: &Path) -> Result<()> {
    let config_path = base_dir.join("heisenberg.toml");

    if config_path.exists() {
        anyhow::bail!("heisenberg.toml already exists");
    }

    // Infer SPA configuration
    let spa_config = infer_spa_config_in_dir(base_dir)?;

    let config = HeisenbergConfig {
        spa: heisenberg::config::toml_config::SpaConfigOrArray::Single(spa_config),
    };

    let toml_content = toml::to_string_pretty(&config)?;
    fs::write(&config_path, toml_content).context("Failed to write heisenberg.toml")?;

    println!("✅ Created heisenberg.toml");
    Ok(())
}

/// Infer SPA configuration from the specified base directory.
///
/// Checks common frontend directory patterns and returns the first match.
pub fn infer_spa_config_in_dir(base_dir: &Path) -> Result<SpaConfig> {
    // Check common frontend directories
    let patterns = [
        ("web", "web/build"),
        ("web", "web/dist"),
        ("frontend", "frontend/build"),
        ("frontend", "frontend/dist"),
        (".", "dist"),
        (".", "build"),
    ];

    for (working_dir, output_dir) in patterns {
        let working_path = base_dir.join(working_dir);
        let package_json = working_path.join("package.json");

        if package_json.exists() {
            // Return relative paths for config file (relative to base_dir)
            let rel_working = if working_dir == "." {
                PathBuf::from(".")
            } else {
                PathBuf::from(format!("./{}", working_dir))
            };
            let rel_output = PathBuf::from(format!("./{}", output_dir));

            return Ok(SpaConfig {
                name: None,
                working_dir: rel_working,
                output_dir: rel_output,
                dev_command: None,
                build_command: None,
                dev_server: None,
            });
        }
    }

    anyhow::bail!("Could not find package.json. Please create heisenberg.toml manually.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Note: init supports root-level package.json unlike build/run
    #[test]
    fn test_infer_supports_root_package_json() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();

        let config = infer_spa_config_in_dir(temp.path()).unwrap();
        assert_eq!(config.working_dir, PathBuf::from("."));
    }

    #[test]
    fn test_run_in_dir_creates_config_file() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("web")).unwrap();
        fs::write(temp.path().join("web/package.json"), "{}").unwrap();

        run_in_dir(temp.path()).unwrap();

        let content = fs::read_to_string(temp.path().join("heisenberg.toml")).unwrap();
        assert!(content.contains("[spa]"));
    }

    #[test]
    fn test_run_in_dir_fails_when_config_exists() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("web")).unwrap();
        fs::write(temp.path().join("web/package.json"), "{}").unwrap();
        fs::write(temp.path().join("heisenberg.toml"), "existing").unwrap();

        let err = run_in_dir(temp.path()).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }
}
