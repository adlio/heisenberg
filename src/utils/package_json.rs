//! Package.json parsing and inference utilities

use crate::error::HeisenbergError;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Package.json data structure
#[derive(Debug, Clone)]
pub struct PackageJson {
    /// Scripts defined in package.json
    pub scripts: HashMap<String, String>,
    /// Package name
    pub name: Option<String>,
    /// Package version
    pub version: Option<String>,
}

/// Infer development configuration from a working directory (app directory)
pub fn infer_from_working_dir(working_dir: &Path) -> Result<InferredConfig, HeisenbergError> {
    let package_json_path = find_package_json(working_dir)?;
    let package_json = parse_package_json(&package_json_path)?;
    let dev_command = infer_dev_command(&package_json);

    let dev_port =
        read_vite_config_port(working_dir).unwrap_or_else(|| infer_dev_port(&package_json));

    Ok(InferredConfig {
        working_dir: working_dir.to_path_buf(),
        package_json_path,
        dev_command,
        dev_port,
        dev_url: format!("http://localhost:{}", dev_port),
    })
}

/// Infer development configuration from a build directory path
pub fn infer_from_build_dir(build_dir: &Path) -> Result<InferredConfig, HeisenbergError> {
    let working_dir = infer_working_dir(build_dir)?;
    infer_from_working_dir(&working_dir)
}

/// Infer output directory from working directory
pub fn infer_output_dir(working_dir: &Path) -> PathBuf {
    for candidate in &["build", "dist", ".next", ".svelte-kit/output"] {
        let path = working_dir.join(candidate);
        if path.exists() {
            return path;
        }
    }
    working_dir.join("build")
}

/// Inferred configuration from package.json
#[derive(Debug, Clone)]
pub struct InferredConfig {
    /// Working directory containing package.json
    pub working_dir: PathBuf,
    /// Path to the package.json file
    pub package_json_path: PathBuf,
    /// Inferred development command
    pub dev_command: Vec<String>,
    /// Inferred development server port
    pub dev_port: u16,
    /// Inferred development server URL
    pub dev_url: String,
}

/// Infer working directory from build directory path
fn infer_working_dir(build_dir: &Path) -> Result<PathBuf, HeisenbergError> {
    let build_dir = build_dir
        .canonicalize()
        .map_err(|e| HeisenbergError::config(
            format!("Cannot resolve build directory: {}", e),
            "• Check if the build directory path exists\n• Ensure you have read permissions\n• Use an absolute path or verify the relative path is correct"
        ))?;

    // Common build directory names to strip
    let build_names = ["dist", "build", "out", "public", "www"];

    if let Some(dir_name) = build_dir.file_name().and_then(|n| n.to_str()) {
        if build_names.contains(&dir_name) {
            if let Some(parent) = build_dir.parent() {
                return Ok(parent.to_path_buf());
            }
        }
    }

    // If not a recognized build directory, use the directory itself
    Ok(build_dir)
}

/// Find package.json by walking up the directory tree
fn find_package_json(start_dir: &Path) -> Result<PathBuf, HeisenbergError> {
    let mut current = start_dir;

    loop {
        let package_json = current.join("package.json");
        if package_json.exists() {
            return Ok(package_json);
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    Err(HeisenbergError::config(
        format!("No package.json found starting from {}", start_dir.display()),
        "• Ensure package.json exists in your frontend directory\n• Check the embed directory path is correct\n• The search looks in the directory and parent directories\n• Create a package.json with 'npm init' if needed"
    ))
}

/// Parse package.json file
fn parse_package_json(path: &Path) -> Result<PackageJson, HeisenbergError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| HeisenbergError::config(
            format!("Cannot read package.json: {}", e),
            "• Check file permissions on package.json\n• Ensure the file exists and is readable\n• Verify the path is correct"
        ))?;

    let json: Value = serde_json::from_str(&content)?;

    let scripts = json
        .get("scripts")
        .and_then(|s| s.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    let name = json.get("name").and_then(|n| n.as_str()).map(String::from);
    let version = json
        .get("version")
        .and_then(|v| v.as_str())
        .map(String::from);

    Ok(PackageJson {
        scripts,
        name,
        version,
    })
}

/// Infer development command from package.json scripts
fn infer_dev_command(package_json: &PackageJson) -> Vec<String> {
    // Priority order for dev commands
    let command_priorities = ["dev", "start", "serve"];

    for cmd in &command_priorities {
        if package_json.scripts.contains_key(*cmd) {
            return vec!["npm".to_string(), "run".to_string(), cmd.to_string()];
        }
    }

    // Look for any script starting with "dev:"
    for name in package_json.scripts.keys() {
        if name.starts_with("dev:") {
            return vec!["npm".to_string(), "run".to_string(), name.clone()];
        }
    }

    // Default fallback
    vec!["npm".to_string(), "run".to_string(), "dev".to_string()]
}

/// Infer development server port from package.json scripts
fn infer_dev_port(package_json: &PackageJson) -> u16 {
    // Look for port numbers in dev scripts
    for script in package_json.scripts.values() {
        if let Some(port) = extract_port_from_script(script) {
            return port;
        }
    }

    // Common defaults based on tools
    for script in package_json.scripts.values() {
        if script.contains("vite") {
            return 5173; // Vite default
        }
        if script.contains("webpack") || script.contains("react-scripts") {
            return 3000; // CRA/Webpack default
        }
        if script.contains("next") {
            return 3000; // Next.js default
        }
    }

    // Final fallback
    5173 // Vite default as most common modern tool
}

/// Try to read port from vite.config.js
fn read_vite_config_port(working_dir: &Path) -> Option<u16> {
    let vite_configs = ["vite.config.js", "vite.config.ts", "vite.config.mjs"];

    for config_name in &vite_configs {
        let config_path = working_dir.join(config_name);
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            // Remove all whitespace and newlines to handle any formatting
            let normalized: String = content.chars().filter(|c| !c.is_whitespace()).collect();

            // Look for "port:" followed by digits
            if let Some(pos) = normalized.find("port:") {
                let after = &normalized[pos + 5..];
                let port_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(port) = port_str.parse::<u16>() {
                    return Some(port);
                }
            }
        }
    }
    None
}

/// Extract port number from a script command
fn extract_port_from_script(script: &str) -> Option<u16> {
    // Look for --port, -p, or PORT= patterns
    let patterns = ["--port ", "--port=", "-p ", "-p=", "PORT="];

    for pattern in &patterns {
        if let Some(pos) = script.find(pattern) {
            let after = &script[pos + pattern.len()..];
            if let Some(port_str) = after.split_whitespace().next() {
                if let Ok(port) = port_str.parse::<u16>() {
                    return Some(port);
                }
            }
        }
    }

    None
}

impl InferredConfig {
    /// Create default configuration when inference fails
    pub fn default_for_dir(build_dir: &Path) -> Self {
        // Try to infer working directory, fallback to parent of build dir
        let working_dir = build_dir.parent().unwrap_or(build_dir).to_path_buf();

        Self {
            working_dir,
            package_json_path: PathBuf::new(), // Empty path indicates no package.json found
            dev_command: vec!["npm".to_string(), "run".to_string(), "dev".to_string()],
            dev_port: 5173, // Vite default
            dev_url: "http://localhost:5173".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ==================== extract_port_from_script tests ====================

    #[test]
    fn test_extract_port_with_double_dash_space() {
        assert_eq!(extract_port_from_script("vite --port 3000"), Some(3000));
    }

    #[test]
    fn test_extract_port_with_double_dash_equals() {
        assert_eq!(extract_port_from_script("vite --port=4000"), Some(4000));
    }

    #[test]
    fn test_extract_port_with_short_flag_space() {
        assert_eq!(extract_port_from_script("vite -p 5000"), Some(5000));
    }

    #[test]
    fn test_extract_port_with_short_flag_equals() {
        assert_eq!(extract_port_from_script("vite -p=6000"), Some(6000));
    }

    #[test]
    fn test_extract_port_with_env_var() {
        assert_eq!(
            extract_port_from_script("PORT=8080 node server.js"),
            Some(8080)
        );
    }

    #[test]
    fn test_extract_port_no_port_specified() {
        assert_eq!(extract_port_from_script("vite"), None);
    }

    #[test]
    fn test_extract_port_invalid_port_number() {
        // Port after flag is not a number
        assert_eq!(extract_port_from_script("vite --port abc"), None);
    }

    #[test]
    fn test_extract_port_in_complex_script() {
        assert_eq!(
            extract_port_from_script("cross-env NODE_ENV=dev vite --port 3001 --host"),
            Some(3001)
        );
    }

    // ==================== infer_dev_command tests ====================

    #[test]
    fn test_infer_dev_command_prefers_dev() {
        let mut scripts = HashMap::new();
        scripts.insert("dev".to_string(), "vite".to_string());
        scripts.insert("start".to_string(), "node server.js".to_string());
        scripts.insert("serve".to_string(), "serve -s build".to_string());

        let pkg = PackageJson {
            scripts,
            name: None,
            version: None,
        };

        assert_eq!(infer_dev_command(&pkg), vec!["npm", "run", "dev"]);
    }

    #[test]
    fn test_infer_dev_command_falls_back_to_start() {
        let mut scripts = HashMap::new();
        scripts.insert("start".to_string(), "node server.js".to_string());
        scripts.insert("serve".to_string(), "serve -s build".to_string());

        let pkg = PackageJson {
            scripts,
            name: None,
            version: None,
        };

        assert_eq!(infer_dev_command(&pkg), vec!["npm", "run", "start"]);
    }

    #[test]
    fn test_infer_dev_command_falls_back_to_serve() {
        let mut scripts = HashMap::new();
        scripts.insert("serve".to_string(), "serve -s build".to_string());

        let pkg = PackageJson {
            scripts,
            name: None,
            version: None,
        };

        assert_eq!(infer_dev_command(&pkg), vec!["npm", "run", "serve"]);
    }

    #[test]
    fn test_infer_dev_command_matches_dev_prefixed() {
        let mut scripts = HashMap::new();
        scripts.insert("dev:client".to_string(), "vite".to_string());

        let pkg = PackageJson {
            scripts,
            name: None,
            version: None,
        };

        assert_eq!(infer_dev_command(&pkg), vec!["npm", "run", "dev:client"]);
    }

    #[test]
    fn test_infer_dev_command_default_fallback() {
        let pkg = PackageJson {
            scripts: HashMap::new(),
            name: None,
            version: None,
        };

        assert_eq!(infer_dev_command(&pkg), vec!["npm", "run", "dev"]);
    }

    // ==================== infer_dev_port tests ====================

    #[test]
    fn test_infer_dev_port_from_script() {
        let mut scripts = HashMap::new();
        scripts.insert("dev".to_string(), "vite --port 4200".to_string());

        let pkg = PackageJson {
            scripts,
            name: None,
            version: None,
        };

        assert_eq!(infer_dev_port(&pkg), 4200);
    }

    #[test]
    fn test_infer_dev_port_vite_default() {
        let mut scripts = HashMap::new();
        scripts.insert("dev".to_string(), "vite".to_string());

        let pkg = PackageJson {
            scripts,
            name: None,
            version: None,
        };

        assert_eq!(infer_dev_port(&pkg), 5173);
    }

    #[test]
    fn test_infer_dev_port_webpack_default() {
        let mut scripts = HashMap::new();
        scripts.insert("dev".to_string(), "webpack serve".to_string());

        let pkg = PackageJson {
            scripts,
            name: None,
            version: None,
        };

        assert_eq!(infer_dev_port(&pkg), 3000);
    }

    #[test]
    fn test_infer_dev_port_react_scripts_default() {
        let mut scripts = HashMap::new();
        scripts.insert("start".to_string(), "react-scripts start".to_string());

        let pkg = PackageJson {
            scripts,
            name: None,
            version: None,
        };

        assert_eq!(infer_dev_port(&pkg), 3000);
    }

    #[test]
    fn test_infer_dev_port_next_default() {
        let mut scripts = HashMap::new();
        scripts.insert("dev".to_string(), "next dev".to_string());

        let pkg = PackageJson {
            scripts,
            name: None,
            version: None,
        };

        assert_eq!(infer_dev_port(&pkg), 3000);
    }

    #[test]
    fn test_infer_dev_port_empty_scripts_fallback() {
        let pkg = PackageJson {
            scripts: HashMap::new(),
            name: None,
            version: None,
        };

        assert_eq!(infer_dev_port(&pkg), 5173);
    }

    // ==================== read_vite_config_port tests ====================

    #[test]
    fn test_read_vite_config_port_js() {
        let temp_dir = TempDir::new().unwrap();
        let config_content = r#"
            export default {
                server: {
                    port: 4000
                }
            }
        "#;
        fs::write(temp_dir.path().join("vite.config.js"), config_content).unwrap();

        assert_eq!(read_vite_config_port(temp_dir.path()), Some(4000));
    }

    #[test]
    fn test_read_vite_config_port_ts() {
        let temp_dir = TempDir::new().unwrap();
        let config_content = r#"
            import { defineConfig } from 'vite'
            export default defineConfig({
                server: {
                    port: 5000
                }
            })
        "#;
        fs::write(temp_dir.path().join("vite.config.ts"), config_content).unwrap();

        assert_eq!(read_vite_config_port(temp_dir.path()), Some(5000));
    }

    #[test]
    fn test_read_vite_config_port_mjs() {
        let temp_dir = TempDir::new().unwrap();
        let config_content = "export default { server: { port: 6000 } }";
        fs::write(temp_dir.path().join("vite.config.mjs"), config_content).unwrap();

        assert_eq!(read_vite_config_port(temp_dir.path()), Some(6000));
    }

    #[test]
    fn test_read_vite_config_port_no_port() {
        let temp_dir = TempDir::new().unwrap();
        let config_content = "export default { plugins: [] }";
        fs::write(temp_dir.path().join("vite.config.js"), config_content).unwrap();

        assert_eq!(read_vite_config_port(temp_dir.path()), None);
    }

    #[test]
    fn test_read_vite_config_port_no_config_file() {
        let temp_dir = TempDir::new().unwrap();
        assert_eq!(read_vite_config_port(temp_dir.path()), None);
    }

    // ==================== infer_output_dir tests ====================

    #[test]
    fn test_infer_output_dir_prefers_build() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join("build")).unwrap();
        fs::create_dir(temp_dir.path().join("dist")).unwrap();

        let result = infer_output_dir(temp_dir.path());
        assert!(result.ends_with("build"));
    }

    #[test]
    fn test_infer_output_dir_falls_back_to_dist() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join("dist")).unwrap();

        let result = infer_output_dir(temp_dir.path());
        assert!(result.ends_with("dist"));
    }

    #[test]
    fn test_infer_output_dir_finds_next() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join(".next")).unwrap();

        let result = infer_output_dir(temp_dir.path());
        assert!(result.ends_with(".next"));
    }

    #[test]
    fn test_infer_output_dir_finds_sveltekit() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join(".svelte-kit/output")).unwrap();

        let result = infer_output_dir(temp_dir.path());
        assert!(result.ends_with("output"));
    }

    #[test]
    fn test_infer_output_dir_default_fallback() {
        let temp_dir = TempDir::new().unwrap();

        let result = infer_output_dir(temp_dir.path());
        assert!(result.ends_with("build"));
    }

    // ==================== infer_working_dir tests ====================

    #[test]
    fn test_infer_working_dir_strips_dist() {
        let temp_dir = TempDir::new().unwrap();
        let dist_dir = temp_dir.path().join("dist");
        fs::create_dir(&dist_dir).unwrap();

        let result = infer_working_dir(&dist_dir).unwrap();
        assert_eq!(
            result.canonicalize().unwrap(),
            temp_dir.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn test_infer_working_dir_strips_build() {
        let temp_dir = TempDir::new().unwrap();
        let build_dir = temp_dir.path().join("build");
        fs::create_dir(&build_dir).unwrap();

        let result = infer_working_dir(&build_dir).unwrap();
        assert_eq!(
            result.canonicalize().unwrap(),
            temp_dir.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn test_infer_working_dir_strips_out() {
        let temp_dir = TempDir::new().unwrap();
        let out_dir = temp_dir.path().join("out");
        fs::create_dir(&out_dir).unwrap();

        let result = infer_working_dir(&out_dir).unwrap();
        assert_eq!(
            result.canonicalize().unwrap(),
            temp_dir.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn test_infer_working_dir_keeps_unrecognized() {
        let temp_dir = TempDir::new().unwrap();
        let custom_dir = temp_dir.path().join("custom");
        fs::create_dir(&custom_dir).unwrap();

        let result = infer_working_dir(&custom_dir).unwrap();
        assert_eq!(
            result.canonicalize().unwrap(),
            custom_dir.canonicalize().unwrap()
        );
    }

    #[test]
    fn test_infer_working_dir_nonexistent() {
        let result = infer_working_dir(Path::new("/nonexistent/path/dist"));
        assert!(result.is_err());
    }

    // ==================== find_package_json tests ====================

    #[test]
    fn test_find_package_json_in_same_dir() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("package.json"), "{}").unwrap();

        let result = find_package_json(temp_dir.path()).unwrap();
        assert!(result.ends_with("package.json"));
    }

    #[test]
    fn test_find_package_json_in_parent() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("sub");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(temp_dir.path().join("package.json"), "{}").unwrap();

        let result = find_package_json(&sub_dir).unwrap();
        assert!(result.ends_with("package.json"));
    }

    #[test]
    fn test_find_package_json_walks_up_tree() {
        let temp_dir = TempDir::new().unwrap();
        let deep_dir = temp_dir.path().join("a/b/c");
        fs::create_dir_all(&deep_dir).unwrap();
        fs::write(temp_dir.path().join("package.json"), "{}").unwrap();

        let result = find_package_json(&deep_dir).unwrap();
        assert!(result.ends_with("package.json"));
    }

    #[test]
    fn test_find_package_json_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let result = find_package_json(temp_dir.path());
        assert!(result.is_err());
    }

    // ==================== parse_package_json tests ====================

    #[test]
    fn test_parse_package_json_full() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"{
            "name": "my-app",
            "version": "1.0.0",
            "scripts": {
                "dev": "vite",
                "build": "vite build"
            }
        }"#;
        let path = temp_dir.path().join("package.json");
        fs::write(&path, content).unwrap();

        let result = parse_package_json(&path).unwrap();
        assert_eq!(result.name, Some("my-app".to_string()));
        assert_eq!(result.version, Some("1.0.0".to_string()));
        assert_eq!(result.scripts.get("dev"), Some(&"vite".to_string()));
    }

    #[test]
    fn test_parse_package_json_minimal() {
        let temp_dir = TempDir::new().unwrap();
        let content = "{}";
        let path = temp_dir.path().join("package.json");
        fs::write(&path, content).unwrap();

        let result = parse_package_json(&path).unwrap();
        assert_eq!(result.name, None);
        assert_eq!(result.version, None);
        assert!(result.scripts.is_empty());
    }

    #[test]
    fn test_parse_package_json_missing_scripts() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"{"name": "test"}"#;
        let path = temp_dir.path().join("package.json");
        fs::write(&path, content).unwrap();

        let result = parse_package_json(&path).unwrap();
        assert!(result.scripts.is_empty());
    }

    #[test]
    fn test_parse_package_json_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let content = "not valid json";
        let path = temp_dir.path().join("package.json");
        fs::write(&path, content).unwrap();

        let result = parse_package_json(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_package_json_nonexistent() {
        let result = parse_package_json(Path::new("/nonexistent/package.json"));
        assert!(result.is_err());
    }

    // ==================== InferredConfig::default_for_dir tests ====================

    #[test]
    fn test_inferred_config_default_for_dir() {
        let temp_dir = TempDir::new().unwrap();
        let build_dir = temp_dir.path().join("build");

        let config = InferredConfig::default_for_dir(&build_dir);

        assert_eq!(config.working_dir, temp_dir.path());
        assert!(config.package_json_path.as_os_str().is_empty());
        assert_eq!(config.dev_command, vec!["npm", "run", "dev"]);
        assert_eq!(config.dev_port, 5173);
        assert_eq!(config.dev_url, "http://localhost:5173");
    }
}
