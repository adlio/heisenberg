//! Configuration file structures for heisenberg.toml

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeisenbergConfig {
    /// Default SPA configuration
    #[serde(default)]
    pub spa: Option<SpaConfig>,
    /// Named SPA configurations
    #[serde(flatten)]
    pub named_spas: HashMap<String, NamedSpaWrapper>,
}

/// Wrapper for named SPA configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedSpaWrapper {
    /// SPA configuration
    pub spa: SpaConfig,
}

/// SPA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaConfig {
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

    /// Get SPA configuration by name (None for default)
    pub fn get_spa(&self, name: Option<&str>) -> Option<&SpaConfig> {
        match name {
            None => self.spa.as_ref(),
            Some(n) => self.named_spas.get(n).map(|w| &w.spa),
        }
    }
}
