//! Static file serving for production mode

use crate::error::HeisenbergError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use std::path::{Path, PathBuf};
use tokio::fs;

type Body = Full<Bytes>;

/// Static file service for serving files from a directory
pub struct StaticFileService {
    base_dir: PathBuf,
    fallback_file: Option<String>,
}

impl StaticFileService {
    /// Create a new static file service
    pub fn new(base_dir: PathBuf, fallback_file: Option<String>) -> Self {
        Self {
            base_dir,
            fallback_file,
        }
    }

    /// Serve a file by path
    pub async fn serve_file(&self, path: &str) -> Result<Response<Body>, HeisenbergError> {
        let clean_path = path.trim_start_matches('/');
        let file_path = if clean_path.is_empty() {
            self.base_dir.join("index.html")
        } else {
            self.base_dir.join(clean_path)
        };

        // Security: prevent path traversal
        let canonical_base = self.base_dir.canonicalize().map_err(|e| {
            HeisenbergError::file_not_found(
                self.base_dir.display().to_string(),
                format!("Base directory error: {}", e),
            )
        })?;

        let canonical_file = match file_path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // File doesn't exist, try fallback
                if let Some(ref fallback) = self.fallback_file {
                    return self.serve_fallback(fallback).await;
                }
                return Err(HeisenbergError::file_not_found(path, "File not found"));
            }
        };

        if !canonical_file.starts_with(&canonical_base) {
            return Err(HeisenbergError::file_not_found(
                path,
                "Path traversal attempt blocked",
            ));
        }

        // Read and serve the file
        match fs::read(&canonical_file).await {
            Ok(contents) => {
                let mime_type = self.detect_mime_type(path);
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", mime_type)
                    .body(Full::new(Bytes::from(contents)))
                    .unwrap())
            }
            Err(_) => {
                if let Some(ref fallback) = self.fallback_file {
                    self.serve_fallback(fallback).await
                } else {
                    Err(HeisenbergError::file_not_found(path, "File not found"))
                }
            }
        }
    }

    async fn serve_fallback(&self, fallback: &str) -> Result<Response<Body>, HeisenbergError> {
        let fallback_path = self.base_dir.join(fallback);
        let contents = fs::read(&fallback_path).await.map_err(|e| {
            HeisenbergError::file_not_found(fallback, format!("Fallback file error: {}", e))
        })?;

        let mime_type = self.detect_mime_type(fallback);
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", mime_type)
            .body(Full::new(Bytes::from(contents)))
            .unwrap())
    }

    /// Detect MIME type from file extension
    fn detect_mime_type(&self, path: &str) -> &'static str {
        let ext = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match ext {
            "html" => "text/html; charset=utf-8",
            "css" => "text/css; charset=utf-8",
            "js" | "mjs" => "application/javascript; charset=utf-8",
            "json" => "application/json",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "ico" => "image/x-icon",
            _ => "application/octet-stream",
        }
    }
}
