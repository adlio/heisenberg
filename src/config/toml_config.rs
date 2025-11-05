//! Configuration file structures for heisenberg.toml

use serde::{Deserialize, Serialize};

use std::path::{Path, PathBuf};

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeisenbergConfig {
    /// SPA configurations (supports both single [spa] and array [[spa]])
    #[serde(default)]
    pub spa: SpaConfigOrArray,
}

/// Supports both single SPA and array of SPAs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum SpaConfigOrArray {
    /// Single SPA configuration
    Single(SpaConfig),
    /// Multiple SPA configurations
    Multiple(Vec<SpaConfig>),
    /// No SPA configuration
    #[default]
    None,
}

/// SPA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaConfig {
    /// Optional name for the SPA (used with embed_spa!("name"))
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Directory containing package.json
    pub working_dir: PathBuf,
    /// Directory containing built assets
    pub output_dir: PathBuf,
    /// Command to start dev server (default: inferred from package.json)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_command: Option<String>,
    /// Command to build assets (default: inferred from package.json)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_command: Option<String>,
    /// Dev server URL (default: inferred from config files)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_server: Option<String>,
}

impl HeisenbergConfig {
    /// Load configuration from file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read config: {}", e))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
    }

    /// Get all SPA configurations as a vector
    pub fn spas(&self) -> Vec<&SpaConfig> {
        match &self.spa {
            SpaConfigOrArray::Single(spa) => vec![spa],
            SpaConfigOrArray::Multiple(spas) => spas.iter().collect(),
            SpaConfigOrArray::None => vec![],
        }
    }
}
