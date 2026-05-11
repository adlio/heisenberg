//! Registry for embedded assets

use crate::error::HeisenbergError;
use crate::services::cache::{etag_for, if_none_match, policy_for_path};
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

/// Serve embedded asset with proper MIME type.
///
/// Equivalent to [`serve_embedded_asset_cached`] with no `If-None-Match`
/// header. Kept for backwards compatibility — new callers should prefer the
/// cached variant so clients can revalidate.
pub fn serve_embedded_asset(
    spa_path: &str,
    file_path: &str,
    fallback: Option<&str>,
) -> Result<Response<Body>, HeisenbergError> {
    serve_embedded_asset_cached(spa_path, file_path, fallback, None)
}

/// Serve an embedded asset with smart HTTP caching headers.
///
/// Adds a strong content-hash `ETag` and a `Cache-Control` header chosen by
/// [`crate::services::cache::policy_for_path`]. If the caller supplies the
/// request's `If-None-Match` value and it matches the asset's ETag, returns
/// `304 Not Modified` with no body.
pub fn serve_embedded_asset_cached(
    spa_path: &str,
    file_path: &str,
    fallback: Option<&str>,
    if_none_match_header: Option<&str>,
) -> Result<Response<Body>, HeisenbergError> {
    let clean_path = file_path.trim_start_matches('/');
    let path_to_serve = if clean_path.is_empty() {
        "index.html"
    } else {
        clean_path
    };

    if let Some(content) = get_embedded_asset(spa_path, path_to_serve) {
        return Ok(build_cached_response(
            spa_path,
            path_to_serve,
            content,
            if_none_match_header,
        ));
    }

    // Try fallback
    if let Some(fallback_path) = fallback {
        if let Some(content) = get_embedded_asset(spa_path, fallback_path) {
            // Use the originally-requested path for cache policy — bare SPA
            // routes should revalidate even when their bytes come from
            // index.html — but key the ETag on the actual file so multiple
            // unknown routes share the same cached digest.
            return Ok(build_cached_response_with_paths(
                spa_path,
                fallback_path,
                path_to_serve,
                content,
                if_none_match_header,
            ));
        }
    }

    Err(HeisenbergError::file_not_found(
        file_path,
        "Embedded asset not found",
    ))
}

fn build_cached_response(
    spa_path: &str,
    path: &str,
    content: Vec<u8>,
    if_none_match_header: Option<&str>,
) -> Response<Body> {
    build_cached_response_with_paths(spa_path, path, path, content, if_none_match_header)
}

fn build_cached_response_with_paths(
    spa_path: &str,
    etag_path: &str,
    policy_path: &str,
    content: Vec<u8>,
    if_none_match_header: Option<&str>,
) -> Response<Body> {
    let etag = etag_for(spa_path, etag_path, &content);
    let policy = policy_for_path(policy_path);

    if let Some(client_etag) = if_none_match_header {
        if if_none_match(client_etag, &etag) {
            return Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .header("etag", &etag)
                .header("cache-control", policy.cache_control())
                .body(Full::new(Bytes::new()))
                .unwrap();
        }
    }

    let mime_type = detect_mime_type(policy_path);
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", mime_type)
        .header("etag", &etag)
        .header("cache-control", policy.cache_control())
        .body(Full::new(Bytes::from(content)))
        .unwrap()
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

    // ==================== smart caching tests ====================

    #[test]
    fn serves_etag_and_cache_control_on_200() {
        register_embedded_assets("cache-spa-1", |path| {
            if path == "app.abc12345.js" {
                Some(b"console.log('hi');".to_vec())
            } else {
                None
            }
        });

        let resp =
            serve_embedded_asset_cached("cache-spa-1", "/app.abc12345.js", None, None).unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let etag = resp.headers().get("etag").expect("etag header missing");
        let etag = etag.to_str().unwrap();
        assert!(etag.starts_with('"') && etag.ends_with('"'));

        let cc = resp
            .headers()
            .get("cache-control")
            .expect("cache-control missing")
            .to_str()
            .unwrap();
        assert!(
            cc.contains("immutable"),
            "expected immutable policy, got {cc}"
        );
    }

    #[test]
    fn html_gets_no_cache_policy() {
        register_embedded_assets("cache-spa-html", |path| {
            if path == "index.html" {
                Some(b"<html></html>".to_vec())
            } else {
                None
            }
        });

        let resp =
            serve_embedded_asset_cached("cache-spa-html", "/index.html", None, None).unwrap();
        let cc = resp
            .headers()
            .get("cache-control")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(cc, "no-cache");
    }

    #[test]
    fn plain_assets_get_short_lived_policy() {
        register_embedded_assets("cache-spa-plain", |path| {
            if path == "favicon.ico" {
                Some(vec![0, 1, 2, 3])
            } else {
                None
            }
        });

        let resp =
            serve_embedded_asset_cached("cache-spa-plain", "/favicon.ico", None, None).unwrap();
        let cc = resp
            .headers()
            .get("cache-control")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(cc.contains("must-revalidate"), "got {cc}");
        assert!(cc.contains("max-age=3600"), "got {cc}");
    }

    #[tokio::test]
    async fn matching_if_none_match_returns_304_without_body() {
        register_embedded_assets("cache-spa-304", |path| {
            if path == "asset.5f3a9b2c.js" {
                Some(b"payload".to_vec())
            } else {
                None
            }
        });

        // First fetch to learn the ETag.
        let first =
            serve_embedded_asset_cached("cache-spa-304", "/asset.5f3a9b2c.js", None, None).unwrap();
        let etag = first
            .headers()
            .get("etag")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let second =
            serve_embedded_asset_cached("cache-spa-304", "/asset.5f3a9b2c.js", None, Some(&etag))
                .unwrap();
        assert_eq!(second.status(), StatusCode::NOT_MODIFIED);
        // 304 responses must still carry the ETag and Cache-Control.
        assert_eq!(
            second.headers().get("etag").unwrap().to_str().unwrap(),
            etag
        );
        assert!(second.headers().get("cache-control").is_some());
        // And must have no body.
        let bytes = http_body_util::BodyExt::collect(second.into_body())
            .await
            .unwrap()
            .to_bytes();
        assert!(bytes.is_empty());
    }

    #[test]
    fn nonmatching_if_none_match_returns_200() {
        register_embedded_assets("cache-spa-mismatch", |path| {
            if path == "main.js" {
                Some(b"hello".to_vec())
            } else {
                None
            }
        });

        let resp = serve_embedded_asset_cached(
            "cache-spa-mismatch",
            "/main.js",
            None,
            Some("\"totally-different\""),
        )
        .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[test]
    fn etag_is_stable_across_requests() {
        register_embedded_assets("cache-spa-stable", |path| {
            if path == "bundle.js" {
                Some(b"contents".to_vec())
            } else {
                None
            }
        });

        let a = serve_embedded_asset_cached("cache-spa-stable", "/bundle.js", None, None).unwrap();
        let b = serve_embedded_asset_cached("cache-spa-stable", "/bundle.js", None, None).unwrap();
        assert_eq!(
            a.headers().get("etag").unwrap(),
            b.headers().get("etag").unwrap()
        );
    }

    #[test]
    fn spa_fallback_uses_no_cache_for_bare_route() {
        register_embedded_assets("cache-spa-fallback", |path| {
            if path == "index.html" {
                Some(b"<html>shell</html>".to_vec())
            } else {
                None
            }
        });

        let resp = serve_embedded_asset_cached(
            "cache-spa-fallback",
            "/dashboard",
            Some("index.html"),
            None,
        )
        .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let cc = resp
            .headers()
            .get("cache-control")
            .unwrap()
            .to_str()
            .unwrap();
        // The requested path (/dashboard) has no extension → HTML-shell, must
        // revalidate so users see new deployments without a hard refresh.
        assert_eq!(cc, "no-cache");
    }
}
