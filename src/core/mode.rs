//! Mode detection for proxy vs embed

/// Operating mode for Heisenberg
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// Proxy mode - forward requests to dev servers
    Proxy,
    /// Embed mode - serve embedded static assets
    Embed,
}

/// Detect the current operating mode
pub fn detect_mode() -> Mode {
    // Check environment variable override first
    if let Ok(mode) = std::env::var("HEISENBERG_MODE") {
        match mode.to_lowercase().as_str() {
            "embed" | "production" | "prod" => return Mode::Embed,
            "proxy" | "development" | "dev" => return Mode::Proxy,
            _ => {} // Fall through to default detection
        }
    }

    // Default to Embed mode
    // Use `cargo heisenberg run` or HEISENBERG_MODE=proxy for development
    Mode::Embed
}
