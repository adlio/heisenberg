//! Static file serving (deprecated - embed mode uses only embedded assets)

use crate::error::HeisenbergError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use rust_embed::RustEmbed;
use std::path::{Path, PathBuf};
use tokio::fs;

type Body = Full<Bytes>;

/// Trait for embedded asset providers
pub trait EmbeddedAssets: RustEmbed {}

/// Static file service for serving files from a directory or embedded assets
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

    /// Serve from embedded assets using rust-embed
    pub fn serve_embedded<A: RustEmbed>(
        &self,
        path: &str,
    ) -> Result<Response<Body>, HeisenbergError> {
        let clean_path = path.trim_start_matches('/');
        let file_path = if clean_path.is_empty() {
            "index.html"
        } else {
            clean_path
        };

        match A::get(file_path) {
            Some(content) => {
                let mime_type = self.detect_mime_type(file_path);
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", mime_type)
                    .body(Full::new(Bytes::from(content.data.to_vec())))
                    .unwrap())
            }
            None => {
                if let Some(ref fallback) = self.fallback_file {
                    if let Some(content) = A::get(fallback) {
                        let mime_type = self.detect_mime_type(fallback);
                        return Ok(Response::builder()
                            .status(StatusCode::OK)
                            .header("content-type", mime_type)
                            .body(Full::new(Bytes::from(content.data.to_vec())))
                            .unwrap());
                    }
                }
                Err(HeisenbergError::file_not_found(path, "File not found"))
            }
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
                let mime_type = self.detect_mime_type(canonical_file.to_str().unwrap_or(path));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ==================== MIME type detection tests ====================

    #[test]
    fn test_mime_type_html() {
        let svc = StaticFileService::new(PathBuf::from("."), None);
        assert_eq!(
            svc.detect_mime_type("index.html"),
            "text/html; charset=utf-8"
        );
    }

    #[test]
    fn test_mime_type_css() {
        let svc = StaticFileService::new(PathBuf::from("."), None);
        assert_eq!(
            svc.detect_mime_type("styles.css"),
            "text/css; charset=utf-8"
        );
    }

    #[test]
    fn test_mime_type_javascript() {
        let svc = StaticFileService::new(PathBuf::from("."), None);
        assert_eq!(
            svc.detect_mime_type("app.js"),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(
            svc.detect_mime_type("module.mjs"),
            "application/javascript; charset=utf-8"
        );
    }

    #[test]
    fn test_mime_type_json() {
        let svc = StaticFileService::new(PathBuf::from("."), None);
        assert_eq!(svc.detect_mime_type("data.json"), "application/json");
    }

    #[test]
    fn test_mime_type_images() {
        let svc = StaticFileService::new(PathBuf::from("."), None);
        assert_eq!(svc.detect_mime_type("logo.png"), "image/png");
        assert_eq!(svc.detect_mime_type("photo.jpg"), "image/jpeg");
        assert_eq!(svc.detect_mime_type("photo.jpeg"), "image/jpeg");
        assert_eq!(svc.detect_mime_type("animation.gif"), "image/gif");
        assert_eq!(svc.detect_mime_type("icon.svg"), "image/svg+xml");
        assert_eq!(svc.detect_mime_type("favicon.ico"), "image/x-icon");
    }

    #[test]
    fn test_mime_type_fonts() {
        let svc = StaticFileService::new(PathBuf::from("."), None);
        assert_eq!(svc.detect_mime_type("font.woff"), "font/woff");
        assert_eq!(svc.detect_mime_type("font.woff2"), "font/woff2");
        assert_eq!(svc.detect_mime_type("font.ttf"), "font/ttf");
    }

    #[test]
    fn test_mime_type_unknown() {
        let svc = StaticFileService::new(PathBuf::from("."), None);
        assert_eq!(svc.detect_mime_type("file.xyz"), "application/octet-stream");
        assert_eq!(
            svc.detect_mime_type("noextension"),
            "application/octet-stream"
        );
    }

    // ==================== serve_file path safety tests ====================

    #[tokio::test]
    async fn test_serve_file_success() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "hello world").unwrap();

        let svc = StaticFileService::new(temp_dir.path().to_path_buf(), None);
        let result = svc.serve_file("/test.txt").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_serve_file_empty_path_serves_index() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("index.html"), "<html>").unwrap();

        let svc = StaticFileService::new(temp_dir.path().to_path_buf(), None);
        let result = svc.serve_file("/").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_serve_file_path_traversal_blocked() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join("www");
        fs::create_dir(&base_dir).unwrap();
        fs::write(base_dir.join("safe.txt"), "safe").unwrap();
        // Create a file outside the base directory
        fs::write(temp_dir.path().join("secret.txt"), "secret").unwrap();

        let svc = StaticFileService::new(base_dir.clone(), None);

        // Try to access file outside base directory with path traversal
        let result = svc.serve_file("/../secret.txt").await;

        // Should fail - either file not found or path traversal blocked
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_serve_file_not_found() {
        let temp_dir = TempDir::new().unwrap();

        let svc = StaticFileService::new(temp_dir.path().to_path_buf(), None);
        let result = svc.serve_file("/nonexistent.txt").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_serve_file_fallback() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("index.html"), "<html>fallback</html>").unwrap();

        let svc = StaticFileService::new(
            temp_dir.path().to_path_buf(),
            Some("index.html".to_string()),
        );
        // Request a non-existent file
        let result = svc.serve_file("/app/dashboard").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_serve_file_no_fallback_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("index.html"), "<html>").unwrap();

        let svc = StaticFileService::new(temp_dir.path().to_path_buf(), None);
        let result = svc.serve_file("/nonexistent").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_serve_file_correct_content_type() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("styles.css"), "body {}").unwrap();

        let svc = StaticFileService::new(temp_dir.path().to_path_buf(), None);
        let result = svc.serve_file("/styles.css").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/css; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn test_serve_file_strips_leading_slash() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("file.txt"), "content").unwrap();

        let svc = StaticFileService::new(temp_dir.path().to_path_buf(), None);

        // Both with and without leading slash should work
        let result1 = svc.serve_file("/file.txt").await;
        let result2 = svc.serve_file("file.txt").await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_serve_file_nested_path() {
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("assets/images");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("logo.png"), "PNG data").unwrap();

        let svc = StaticFileService::new(temp_dir.path().to_path_buf(), None);
        let result = svc.serve_file("/assets/images/logo.png").await;

        assert!(result.is_ok());
    }

    // ==================== StaticFileService construction tests ====================

    #[test]
    fn test_new_with_fallback() {
        let svc = StaticFileService::new(PathBuf::from("./dist"), Some("index.html".to_string()));
        assert_eq!(svc.base_dir, PathBuf::from("./dist"));
        assert_eq!(svc.fallback_file, Some("index.html".to_string()));
    }

    #[test]
    fn test_new_without_fallback() {
        let svc = StaticFileService::new(PathBuf::from("./public"), None);
        assert_eq!(svc.base_dir, PathBuf::from("./public"));
        assert_eq!(svc.fallback_file, None);
    }
}
