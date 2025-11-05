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
