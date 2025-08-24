//! Error types for Heisenberg

use thiserror::Error;

/// Main error type for Heisenberg operations
#[derive(Error, Debug)]
pub enum HeisenbergError {
    /// Configuration error with troubleshooting hints
    #[error("Configuration error: {message}\n\nTroubleshooting:\n{hint}")]
    Config {
        /// The error message
        message: String,
        /// Troubleshooting hint
        hint: String,
    },

    /// File not found with helpful suggestions
    #[error("File not found: {path}\n\nTroubleshooting:\n{hint}")]
    FileNotFound {
        /// The file path that was not found
        path: String,
        /// Troubleshooting hint
        hint: String,
    },

    /// No route matched the request
    #[error("No route matched the request")]
    NoRouteMatch,

    /// Process management error with actionable guidance
    #[error("Process error: {message}\n\nTroubleshooting:\n{hint}")]
    Process {
        /// The error message
        message: String,
        /// Troubleshooting hint
        hint: String,
    },

    /// Health check failure with debugging info
    #[error("Health check failed: {message}\n\nTroubleshooting:\n{hint}")]
    HealthCheck {
        /// The error message
        message: String,
        /// Troubleshooting hint
        hint: String,
    },

    /// Proxy request failed
    #[error("Proxy request failed: {0}\n\nTroubleshooting:\n• Check if the frontend dev server is running\n• Verify the proxy URL is correct\n• Ensure the dev server accepts connections from localhost")]
    ProxyError(#[from] reqwest::Error),

    /// HTTP error
    #[error("HTTP error: {0}")]
    HttpError(#[from] hyper::Error),

    /// IO error with context
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Package.json parsing failed with helpful guidance
    #[error("Package.json parsing failed: {0}\n\nTroubleshooting:\n• Ensure package.json exists and contains valid JSON\n• Check for syntax errors (missing commas, quotes, etc.)\n• Verify the file is readable")]
    PackageJsonParse(#[from] serde_json::Error),
}

impl HeisenbergError {
    /// Create a configuration error with helpful troubleshooting
    pub fn config(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            hint: hint.into(),
        }
    }

    /// Create a file not found error with suggestions
    pub fn file_not_found(path: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::FileNotFound {
            path: path.into(),
            hint: hint.into(),
        }
    }

    /// Create a process error with troubleshooting guidance
    pub fn process(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::Process {
            message: message.into(),
            hint: hint.into(),
        }
    }

    /// Create a health check error with debugging info
    pub fn health_check(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::HealthCheck {
            message: message.into(),
            hint: hint.into(),
        }
    }
}
