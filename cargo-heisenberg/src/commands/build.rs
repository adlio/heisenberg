use anyhow::{Context, Result};
use heisenberg::config::{HeisenbergConfig, SpaConfig};
use std::path::Path;
use std::process::Command;

use super::infer;

/// Run the build command in the current directory.
pub fn run(cargo_args: Vec<String>) -> Result<()> {
    run_in_dir(Path::new("."), cargo_args)
}

/// Run the build command in the specified directory.
pub fn run_in_dir(base_dir: &Path, cargo_args: Vec<String>) -> Result<()> {
    let config_path = base_dir.join("heisenberg.toml");

    // Try to load config, or use smart defaults
    let spas = if let Ok(config) = HeisenbergConfig::from_file(&config_path) {
        let spa_configs = config.spas();
        if spa_configs.is_empty() {
            anyhow::bail!("No SPA configurations found in heisenberg.toml");
        }
        spa_configs.into_iter().cloned().collect::<Vec<_>>()
    } else {
        vec![infer_spa_config(base_dir)?]
    };

    // Build all SPAs
    for (idx, spa) in spas.iter().enumerate() {
        if spas.len() > 1 {
            println!("📦 Building SPA {} at {:?}", idx + 1, spa.working_dir);
        }
        build_spa(spa, base_dir)?;
    }

    // Run cargo build
    println!("🦀 Running cargo build...");
    let status = Command::new("cargo")
        .arg("build")
        .args(&cargo_args)
        .current_dir(base_dir)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("cargo build failed");
    }

    println!("✅ Build complete");
    Ok(())
}

/// Build a single SPA by installing dependencies and running the build command.
fn build_spa(spa: &SpaConfig, base_dir: &Path) -> Result<()> {
    let working_dir = base_dir.join(&spa.working_dir);

    // Check if node_modules exists, run npm install if not
    let node_modules = working_dir.join("node_modules");
    if !node_modules.exists() {
        println!("📦 Installing dependencies...");
        let install_status = Command::new("npm")
            .arg("install")
            .current_dir(&working_dir)
            .status()
            .context("Failed to run npm install")?;

        if !install_status.success() {
            anyhow::bail!("npm install failed");
        }
    }

    let build_cmd = spa.build_command.as_deref().unwrap_or("npm run build");

    println!("🏗️  Building frontend: {}", build_cmd);

    let parts: Vec<&str> = build_cmd.split_whitespace().collect();
    let status = Command::new(parts[0])
        .args(&parts[1..])
        .current_dir(&working_dir)
        .status()
        .context("Failed to run build command")?;

    if !status.success() {
        anyhow::bail!("Frontend build failed");
    }

    Ok(())
}

fn infer_spa_config(base_dir: &Path) -> Result<SpaConfig> {
    let working_dir = infer::find_frontend_dir(base_dir)?;
    let abs_working_dir = base_dir.join(&working_dir);
    let output_dir = infer::find_output_dir(&abs_working_dir);
    let build_command = infer::infer_build_command(&abs_working_dir);

    Ok(SpaConfig {
        name: None,
        working_dir,
        output_dir: output_dir
            .strip_prefix(base_dir)
            .unwrap_or(&output_dir)
            .to_path_buf(),
        dev_command: None,
        build_command,
        dev_server: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_infer_spa_config_sets_working_and_output_dir() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("web")).unwrap();
        fs::write(temp.path().join("web/package.json"), "{}").unwrap();

        let config = infer_spa_config(temp.path()).unwrap();
        assert_eq!(config.working_dir, PathBuf::from("./web"));
        assert!(config.output_dir.starts_with("./web") || config.output_dir.starts_with("web"));
    }

    #[test]
    fn test_infer_spa_config_includes_build_command() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("web")).unwrap();
        fs::write(
            temp.path().join("web/package.json"),
            r#"{"scripts":{"build":"vite build"}}"#,
        )
        .unwrap();

        let config = infer_spa_config(temp.path()).unwrap();
        assert_eq!(config.build_command, Some("npm run build".to_string()));
    }

    #[test]
    fn test_infer_spa_config_no_dev_fields() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("web")).unwrap();
        fs::write(temp.path().join("web/package.json"), "{}").unwrap();

        let config = infer_spa_config(temp.path()).unwrap();
        assert!(config.dev_command.is_none());
        assert!(config.dev_server.is_none());
    }

    #[test]
    fn test_build_spa_runs_build_command() {
        let temp = TempDir::new().unwrap();
        let web = temp.path().join("web");
        fs::create_dir_all(&web).unwrap();
        fs::create_dir_all(web.join("node_modules")).unwrap(); // skip npm install
        fs::write(web.join("package.json"), "{}").unwrap();

        let config = SpaConfig {
            name: None,
            working_dir: PathBuf::from("./web"),
            output_dir: PathBuf::from("./web/build"),
            dev_command: None,
            build_command: Some("echo done".to_string()),
            dev_server: None,
        };

        let result = build_spa(&config, temp.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_spa_fails_on_bad_command() {
        let temp = TempDir::new().unwrap();
        let web = temp.path().join("web");
        fs::create_dir_all(&web).unwrap();
        fs::create_dir_all(web.join("node_modules")).unwrap();
        fs::write(web.join("package.json"), "{}").unwrap();

        let config = SpaConfig {
            name: None,
            working_dir: PathBuf::from("./web"),
            output_dir: PathBuf::from("./web/build"),
            dev_command: None,
            build_command: Some("false".to_string()), // exits with code 1
            dev_server: None,
        };

        let result = build_spa(&config, temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_build_spa_runs_npm_install_when_no_node_modules() {
        let temp = TempDir::new().unwrap();
        let web = temp.path().join("web");
        fs::create_dir_all(&web).unwrap();
        // No node_modules - should trigger npm install
        // Empty deps so install is fast
        fs::write(
            web.join("package.json"),
            r#"{"scripts":{"build":"echo built"}}"#,
        )
        .unwrap();

        let config = SpaConfig {
            name: None,
            working_dir: PathBuf::from("./web"),
            output_dir: PathBuf::from("./web/build"),
            dev_command: None,
            build_command: Some("echo done".to_string()),
            dev_server: None,
        };

        let result = build_spa(&config, temp.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_spa_uses_default_command() {
        let temp = TempDir::new().unwrap();
        let web = temp.path().join("web");
        fs::create_dir_all(&web).unwrap();
        fs::create_dir_all(web.join("node_modules")).unwrap();
        // package.json with a build script that just echoes
        fs::write(
            web.join("package.json"),
            r#"{"scripts":{"build":"echo built"}}"#,
        )
        .unwrap();

        let config = SpaConfig {
            name: None,
            working_dir: PathBuf::from("./web"),
            output_dir: PathBuf::from("./web/build"),
            dev_command: None,
            build_command: None, // should default to "npm run build"
            dev_server: None,
        };

        let result = build_spa(&config, temp.path());
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Requires npm/node and clean SvelteKit state - run manually
    fn test_run_in_dir_with_example() {
        let example_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("examples/axum-sveltekit");

        let result = run_in_dir(&example_dir, vec!["--release".to_string()]);
        assert!(result.is_ok());
    }
}
