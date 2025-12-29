//! Configuration file structures for heisenberg.toml

use serde::{Deserialize, Serialize};

use std::path::{Path, PathBuf};

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeisenbergConfig {
    /// SPA configurations (supports both single `[spa]` and array `[[spa]]`)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ==================== SpaConfigOrArray deserialization tests ====================

    #[test]
    fn test_deserialize_single_spa() {
        let toml_content = r#"
            [spa]
            working_dir = "./web"
            output_dir = "./web/dist"
        "#;

        let config: HeisenbergConfig = toml::from_str(toml_content).unwrap();
        let spas = config.spas();
        assert_eq!(spas.len(), 1);
        assert_eq!(spas[0].working_dir, PathBuf::from("./web"));
        assert_eq!(spas[0].output_dir, PathBuf::from("./web/dist"));
    }

    #[test]
    fn test_deserialize_multiple_spas() {
        let toml_content = r#"
            [[spa]]
            name = "admin"
            working_dir = "./admin"
            output_dir = "./admin/dist"

            [[spa]]
            name = "client"
            working_dir = "./client"
            output_dir = "./client/dist"
        "#;

        let config: HeisenbergConfig = toml::from_str(toml_content).unwrap();
        let spas = config.spas();
        assert_eq!(spas.len(), 2);
        assert_eq!(spas[0].name, Some("admin".to_string()));
        assert_eq!(spas[1].name, Some("client".to_string()));
    }

    #[test]
    fn test_deserialize_no_spa() {
        let toml_content = "";

        let config: HeisenbergConfig = toml::from_str(toml_content).unwrap();
        let spas = config.spas();
        assert!(spas.is_empty());
    }

    #[test]
    fn test_deserialize_spa_with_all_optional_fields() {
        let toml_content = r#"
            [spa]
            name = "my-app"
            working_dir = "./frontend"
            output_dir = "./frontend/build"
            dev_command = "npm run dev"
            build_command = "npm run build"
            dev_server = "http://localhost:3000"
        "#;

        let config: HeisenbergConfig = toml::from_str(toml_content).unwrap();
        let spas = config.spas();
        assert_eq!(spas.len(), 1);
        let spa = spas[0];
        assert_eq!(spa.name, Some("my-app".to_string()));
        assert_eq!(spa.dev_command, Some("npm run dev".to_string()));
        assert_eq!(spa.build_command, Some("npm run build".to_string()));
        assert_eq!(spa.dev_server, Some("http://localhost:3000".to_string()));
    }

    #[test]
    fn test_deserialize_spa_with_only_required_fields() {
        let toml_content = r#"
            [spa]
            working_dir = "./web"
            output_dir = "./web/dist"
        "#;

        let config: HeisenbergConfig = toml::from_str(toml_content).unwrap();
        let spas = config.spas();
        assert_eq!(spas.len(), 1);
        let spa = spas[0];
        assert_eq!(spa.name, None);
        assert_eq!(spa.dev_command, None);
        assert_eq!(spa.build_command, None);
        assert_eq!(spa.dev_server, None);
    }

    // ==================== HeisenbergConfig::from_file tests ====================

    #[test]
    fn test_from_file_valid() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("heisenberg.toml");
        let content = r#"
            [spa]
            working_dir = "./web"
            output_dir = "./web/dist"
        "#;
        fs::write(&config_path, content).unwrap();

        let config = HeisenbergConfig::from_file(&config_path).unwrap();
        assert_eq!(config.spas().len(), 1);
    }

    #[test]
    fn test_from_file_nonexistent() {
        let result = HeisenbergConfig::from_file("/nonexistent/heisenberg.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read config"));
    }

    #[test]
    fn test_from_file_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("heisenberg.toml");
        fs::write(&config_path, "not valid toml [[[").unwrap();

        let result = HeisenbergConfig::from_file(&config_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse config"));
    }

    #[test]
    fn test_from_file_missing_required_field() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("heisenberg.toml");
        // Missing output_dir which is required
        let content = r#"
            [spa]
            working_dir = "./web"
        "#;
        fs::write(&config_path, content).unwrap();

        let result = HeisenbergConfig::from_file(&config_path);
        assert!(result.is_err());
    }

    // ==================== SpaConfigOrArray default tests ====================

    #[test]
    fn test_spa_config_or_array_default() {
        let default = SpaConfigOrArray::default();
        match default {
            SpaConfigOrArray::None => {}
            _ => panic!("Default should be None variant"),
        }
    }

    // ==================== Serialization tests ====================

    #[test]
    fn test_serialize_single_spa() {
        let config = HeisenbergConfig {
            spa: SpaConfigOrArray::Single(SpaConfig {
                name: Some("test".to_string()),
                working_dir: PathBuf::from("./web"),
                output_dir: PathBuf::from("./web/dist"),
                dev_command: None,
                build_command: None,
                dev_server: None,
            }),
        };

        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("working_dir"));
        assert!(toml_str.contains("output_dir"));
    }

    #[test]
    fn test_roundtrip_serialization() {
        let original = HeisenbergConfig {
            spa: SpaConfigOrArray::Single(SpaConfig {
                name: Some("my-spa".to_string()),
                working_dir: PathBuf::from("./frontend"),
                output_dir: PathBuf::from("./frontend/dist"),
                dev_command: Some("npm run dev".to_string()),
                build_command: Some("npm run build".to_string()),
                dev_server: Some("http://localhost:5173".to_string()),
            }),
        };

        let toml_str = toml::to_string(&original).unwrap();
        let deserialized: HeisenbergConfig = toml::from_str(&toml_str).unwrap();

        let spas = deserialized.spas();
        assert_eq!(spas.len(), 1);
        assert_eq!(spas[0].name, Some("my-spa".to_string()));
        assert_eq!(spas[0].working_dir, PathBuf::from("./frontend"));
    }
}
