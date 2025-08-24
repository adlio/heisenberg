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

/// Infer development configuration from a build directory path
pub fn infer_from_build_dir(build_dir: &Path) -> Result<InferredConfig, HeisenbergError> {
    let working_dir = infer_working_dir(build_dir)?;
    let package_json_path = find_package_json(&working_dir)?;
    let package_json = parse_package_json(&package_json_path)?;
    let dev_command = infer_dev_command(&package_json);
    let dev_port = infer_dev_port(&package_json);

    Ok(InferredConfig {
        working_dir,
        package_json_path,
        dev_command,
        dev_port,
        dev_url: format!("http://localhost:{}", dev_port),
    })
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
        if let Some(script) = package_json.scripts.get(*cmd) {
            return parse_command(script);
        }
    }

    // Look for any script starting with "dev:"
    for (name, script) in &package_json.scripts {
        if name.starts_with("dev:") {
            return parse_command(script);
        }
    }

    // Default fallback
    vec!["npm".to_string(), "run".to_string(), "dev".to_string()]
}

/// Parse a command string into arguments
fn parse_command(command: &str) -> Vec<String> {
    // Simple parsing - split on whitespace
    // TODO: Handle quoted arguments properly
    command.split_whitespace().map(String::from).collect()
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
