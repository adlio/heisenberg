use anyhow::{Context, Result};
use heisenberg::config::HeisenbergConfig;

use std::process::Command;

pub fn run(cargo_args: Vec<String>) -> Result<()> {
    let config = HeisenbergConfig::from_file("heisenberg.toml")
        .map_err(|e| anyhow::anyhow!("{}", e))
        .context("Failed to read heisenberg.toml. Run 'cargo heisenberg init' first.")?;

    // Build default SPA if present
    if let Some(spa) = &config.spa {
        build_spa(spa)?;
    }

    // Build named SPAs
    for (name, wrapper) in &config.named_spas {
        println!("📦 Building SPA: {}", name);
        build_spa(&wrapper.spa)?;
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
