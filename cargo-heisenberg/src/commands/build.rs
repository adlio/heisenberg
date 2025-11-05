use anyhow::{Context, Result};
use heisenberg::config::HeisenbergConfig;

use std::process::Command;

pub fn run(cargo_args: Vec<String>) -> Result<()> {
    // Try to load config, or use smart defaults
    if let Ok(config) = HeisenbergConfig::from_file("heisenberg.toml") {
        // Build default SPA if present
        if let Some(spa) = &config.spa {
            build_spa(spa)?;
        }

        // Build named SPAs
        for (name, wrapper) in &config.named_spas {
            println!("📦 Building SPA: {}", name);
            build_spa(&wrapper.spa)?;
        }
    } else {
        // Smart defaults: infer from project structure
        let spa = infer_spa_config()?;
        build_spa(&spa)?;
    }

    // Run cargo build
    println!("🦀 Running cargo build...");
    let status = Command::new("cargo")
        .arg("build")
        .args(&cargo_args)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("cargo build failed");
    }

    println!("✅ Build complete");
    Ok(())
}

fn build_spa(spa: &heisenberg::config::SpaConfig) -> Result<()> {
    // Check if node_modules exists, run npm install if not
    let node_modules = spa.working_dir.join("node_modules");
    if !node_modules.exists() {
        println!("📦 Installing dependencies...");
        let install_status = Command::new("npm")
            .arg("install")
            .current_dir(&spa.working_dir)
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
        .current_dir(&spa.working_dir)
        .status()
        .context("Failed to run build command")?;

    if !status.success() {
        anyhow::bail!("Frontend build failed");
    }

    Ok(())
}

fn infer_spa_config() -> Result<heisenberg::config::SpaConfig> {
    use std::path::{Path, PathBuf};

    // Look for ./web or ./frontend
    let working_dir = if Path::new("./web/package.json").exists() {
        PathBuf::from("./web")
    } else if Path::new("./frontend/package.json").exists() {
        PathBuf::from("./frontend")
    } else {
        anyhow::bail!(
            "No frontend found. Create heisenberg.toml or add ./web/package.json or ./frontend/package.json"
        );
    };

    // Infer output directory
    let output_dir = ["build", "dist", ".next", ".svelte-kit/output"]
        .iter()
        .map(|d| working_dir.join(d))
        .find(|p| p.exists())
        .unwrap_or_else(|| working_dir.join("build"));

    // Infer build command from package.json
    let build_command = if let Ok(package_json_content) =
        std::fs::read_to_string(working_dir.join("package.json"))
    {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&package_json_content) {
            json.get("scripts")
                .and_then(|s| s.as_object())
                .and_then(|scripts| {
                    // Try common build script names
                    ["build", "build:prod", "build:production"]
                        .iter()
                        .find(|name| scripts.contains_key(**name))
                        .map(|name| format!("npm run {}", name))
                })
        } else {
            None
        }
    } else {
        None
    };

    Ok(heisenberg::config::SpaConfig {
        working_dir,
        output_dir,
        dev_command: None,
        build_command,
        dev_server: None,
    })
}
