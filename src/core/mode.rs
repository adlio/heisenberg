//! Mode detection for development vs production

/// Operating mode for Heisenberg
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// Development mode - proxy to dev servers
    Development,
    /// Production mode - serve embedded assets
    Production,
}

/// Detect the current operating mode
pub fn detect_mode() -> Mode {
    // Check environment variable override first
    if let Ok(mode) = std::env::var("HEISENBERG_MODE") {
        match mode.to_lowercase().as_str() {
            "production" | "prod" | "embed" => return Mode::Production,
            "development" | "dev" | "proxy" => return Mode::Development,
            _ => {} // Fall through to default detection
        }
    }

    // Check if we're in debug or release mode
    if cfg!(debug_assertions) {
        Mode::Development
    } else {
        Mode::Production
    }
}
