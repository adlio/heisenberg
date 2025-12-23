//! Registry for embedded assets

use crate::error::HeisenbergError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

type Body = Full<Bytes>;
type AssetGetter = Arc<dyn Fn(&str) -> Option<Vec<u8>> + Send + Sync>;

/// Global registry for embedded assets by SPA path
static EMBED_REGISTRY: once_cell::sync::Lazy<RwLock<HashMap<String, AssetGetter>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

/// Register embedded assets for a specific SPA path
pub fn register_embedded_assets<F>(spa_path: &str, getter: F)
where
    F: Fn(&str) -> Option<Vec<u8>> + Send + Sync + 'static,
{
    let mut registry = EMBED_REGISTRY.write().unwrap();
    registry.insert(spa_path.to_string(), Arc::new(getter));
}

/// Get embedded asset for a SPA path
pub fn get_embedded_asset(spa_path: &str, file_path: &str) -> Option<Vec<u8>> {
    let registry = EMBED_REGISTRY.read().unwrap();
    registry.get(spa_path).and_then(|getter| getter(file_path))
}

/// Serve embedded asset with proper MIME type
pub fn serve_embedded_asset(
    spa_path: &str,
    file_path: &str,
    fallback: Option<&str>,
) -> Result<Response<Body>, HeisenbergError> {
    let clean_path = file_path.trim_start_matches('/');
    let path_to_serve = if clean_path.is_empty() {
        "index.html"
    } else {
        clean_path
    };

    if let Some(content) = get_embedded_asset(spa_path, path_to_serve) {
        let mime_type = detect_mime_type(path_to_serve);
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", mime_type)
            .body(Full::new(Bytes::from(content)))
            .unwrap());
    }

    // Try fallback
    if let Some(fallback_path) = fallback {
        if let Some(content) = get_embedded_asset(spa_path, fallback_path) {
            let mime_type = detect_mime_type(fallback_path);
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", mime_type)
                .body(Full::new(Bytes::from(content)))
                .unwrap());
        }
    }

    Err(HeisenbergError::file_not_found(
        file_path,
        "Embedded asset not found",
    ))
}

fn detect_mime_type(path: &str) -> &'static str {
    let ext = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_mime_type() {
        let cases = [
            // HTML (case-insensitive)
            ("index.html", "text/html; charset=utf-8"),
            ("page.HTML", "text/html; charset=utf-8"),
            ("mixed.HtMl", "text/html; charset=utf-8"),
            // CSS (case-insensitive)
            ("style.css", "text/css; charset=utf-8"),
            ("app.CSS", "text/css; charset=utf-8"),
            // JavaScript
            ("app.js", "application/javascript; charset=utf-8"),
            ("bundle.JS", "application/javascript; charset=utf-8"),
            ("module.mjs", "application/javascript; charset=utf-8"),
            // JSON
            ("data.json", "application/json"),
            ("config.JSON", "application/json"),
            // Images
            ("logo.png", "image/png"),
            ("logo.PNG", "image/png"),
            ("photo.jpg", "image/jpeg"),
            ("photo.JPG", "image/jpeg"),
            ("photo.jpeg", "image/jpeg"),
            ("animation.gif", "image/gif"),
            ("icon.svg", "image/svg+xml"),
            ("favicon.ico", "image/x-icon"),
            // Fonts
            ("font.woff", "font/woff"),
            ("font.woff2", "font/woff2"),
            ("font.ttf", "font/ttf"),
            // Unknown/fallback
            ("file.xyz", "application/octet-stream"),
            ("noextension", "application/octet-stream"),
            ("", "application/octet-stream"),
            // Paths with directories
            ("assets/js/app.js", "application/javascript; charset=utf-8"),
            ("/deep/nested/path/style.css", "text/css; charset=utf-8"),
        ];

        for (path, expected_mime) in cases {
            assert_eq!(
                detect_mime_type(path),
                expected_mime,
                "MIME type mismatch for path: {}",
                path
            );
        }
    }

    #[test]
    fn test_register_and_get_embedded_asset() {
        let test_content = b"test content".to_vec();
        let test_content_clone = test_content.clone();

        register_embedded_assets("test-spa-path", move |path| {
            if path == "test.txt" {
                Some(test_content_clone.clone())
            } else {
                None
            }
        });

        // Should retrieve registered asset
        let result = get_embedded_asset("test-spa-path", "test.txt");
        assert_eq!(result, Some(test_content));

        // Should return None for non-existent file
        let result = get_embedded_asset("test-spa-path", "nonexistent.txt");
        assert_eq!(result, None);

        // Should return None for non-existent spa path
        let result = get_embedded_asset("nonexistent-spa", "test.txt");
        assert_eq!(result, None);
    }

    #[test]
    fn test_serve_embedded_asset_success() {
        let html_content = b"<html>test</html>".to_vec();
        let html_clone = html_content.clone();

        register_embedded_assets("serve-test-spa", move |path| {
            if path == "index.html" {
                Some(html_clone.clone())
            } else {
                None
            }
        });

        let result = serve_embedded_asset("serve-test-spa", "/index.html", None);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );
    }

    #[test]
    fn test_serve_embedded_asset_empty_path_serves_index() {
        let html_content = b"<html>index</html>".to_vec();
        let html_clone = html_content.clone();

        register_embedded_assets("index-test-spa", move |path| {
            if path == "index.html" {
                Some(html_clone.clone())
            } else {
                None
            }
        });

        // Empty path should serve index.html
        let result = serve_embedded_asset("index-test-spa", "", None);
        assert!(result.is_ok());

        // Root path should serve index.html
        let result = serve_embedded_asset("index-test-spa", "/", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_serve_embedded_asset_fallback() {
        let fallback_content = b"<html>fallback</html>".to_vec();
        let fallback_clone = fallback_content.clone();

        register_embedded_assets("fallback-test-spa", move |path| {
            if path == "index.html" {
                Some(fallback_clone.clone())
            } else {
                None
            }
        });

        // Non-existent file with fallback should serve fallback
        let result = serve_embedded_asset(
            "fallback-test-spa",
            "/nonexistent/route",
            Some("index.html"),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_serve_embedded_asset_not_found() {
        register_embedded_assets("notfound-test-spa", |_| None);

        // No fallback, should error
        let result = serve_embedded_asset("notfound-test-spa", "/missing.txt", None);
        assert!(result.is_err());

        // With fallback that also doesn't exist
        let result = serve_embedded_asset(
            "notfound-test-spa",
            "/missing.txt",
            Some("also-missing.html"),
        );
        assert!(result.is_err());
    }
}
