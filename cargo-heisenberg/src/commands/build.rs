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
        // Smart defaults: look for ./web or ./frontend
        use std::path::Path;
        let working_dir = if Path::new("./web/package.json").exists() {
            "./web"
        } else if Path::new("./frontend/package.json").exists() {
            "./frontend"
        } else {
            anyhow::bail!("No frontend found. Create heisenberg.toml or add ./web/package.json or ./frontend/package.json");
        };

        let spa = heisenberg::config::SpaConfig {
            working_dir: working_dir.into(),
            output_dir: format!("{}/build", working_dir).into(),
            dev_command: None,
            build_command: None,
            dev_server: None,
        };

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
