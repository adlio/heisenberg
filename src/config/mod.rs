//! Configuration file support

/// TOML configuration structures
pub mod toml_config;

pub use toml_config::{HeisenbergConfig, SpaConfig};

use std::path::PathBuf;

/// Resolve output directory from heisenberg.toml or fallback to defaults
pub fn resolve_output_dir(_name: Option<&str>) -> String {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("heisenberg.toml");

    if let Ok(config) = HeisenbergConfig::from_file(&config_path) {
        let spas = config.spas();
        if let Some(spa) = spas.first() {
            return spa.output_dir.to_string_lossy().to_string();
        }
    }

    // Fallback: infer from common patterns
    for candidate in &[
        "./web/build",
        "./web/dist",
        "./frontend/build",
        "./frontend/dist",
        "./dist",
        "./build",
    ] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(candidate);
        if path.exists() {
            return candidate.to_string();
        }
    }

    panic!("Could not find output directory. Create heisenberg.toml or use standard paths (./dist, ./build, etc.)");
}
