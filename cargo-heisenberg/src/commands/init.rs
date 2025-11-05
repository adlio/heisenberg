use anyhow::{Context, Result};
use heisenberg::config::{HeisenbergConfig, SpaConfig};
use std::fs;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    let config_path = PathBuf::from("heisenberg.toml");

    if config_path.exists() {
        anyhow::bail!("heisenberg.toml already exists");
    }

    // Infer SPA configuration
    let spa_config = infer_spa_config()?;

    let config = HeisenbergConfig {
        spa: Some(spa_config),
        named_spas: Default::default(),
    };

    let toml_content = toml::to_string_pretty(&config)?;
    fs::write(&config_path, toml_content).context("Failed to write heisenberg.toml")?;

    println!("✅ Created heisenberg.toml");
    Ok(())
}

fn infer_spa_config() -> Result<SpaConfig> {
    // Check common frontend directories
    for (working_dir, output_dir) in &[
        ("./web", "./web/build"),
        ("./web", "./web/dist"),
        ("./frontend", "./frontend/build"),
        ("./frontend", "./frontend/dist"),
        (".", "./dist"),
        (".", "./build"),
    ] {
        let working_path = PathBuf::from(working_dir);
        let package_json = working_path.join("package.json");

        if package_json.exists() {
            return Ok(SpaConfig {
                working_dir: PathBuf::from(working_dir),
                output_dir: PathBuf::from(output_dir),
                dev_command: None,
                build_command: None,
                dev_server: None,
            });
        }
    }

    anyhow::bail!("Could not find package.json. Please create heisenberg.toml manually.");
}
