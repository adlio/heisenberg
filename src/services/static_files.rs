//! Static file serving for production mode

use crate::error::HeisenbergError;
use hyper::{Response, StatusCode};
// use rust_embed::RustEmbed; // Will be used when we add actual embedded assets
use std::path::Path;

/// Static file service using embedded assets
pub struct StaticFileService {
    #[allow(dead_code)] // Will be used when we add actual embedded assets
    fallback_file: Option<String>,
}

impl StaticFileService {
    /// Create a new static file service
    pub fn new(fallback_file: Option<String>) -> Self {
        Self { fallback_file }
    }

    /// Serve a file by path
    pub fn serve_file(&self, path: &str) -> Result<Response<String>, HeisenbergError> {
        // For now, just return a simple response
        // Will be enhanced with actual rust-embed integration
        if path == "/" || path == "/index.html" {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/html")
                .body("<html><body><h1>Heisenberg Static Server</h1></body></html>".to_string())
                .unwrap())
        } else {
            Err(HeisenbergError::file_not_found(
                path,
                "• Check if the file exists in the embedded assets\n• Verify the frontend build completed successfully\n• Ensure the embed directory path is correct\n• For SPAs, missing files should fall back to index.html"
            ))
        }
    }

    /// Detect MIME type from file extension
    #[allow(dead_code)] // Will be used when we add actual file serving
    fn detect_mime_type(&self, path: &str) -> &'static str {
        let ext = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match ext {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "svg" => "image/svg+xml",
            _ => "application/octet-stream",
        }
    }
}
