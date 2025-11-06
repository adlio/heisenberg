//! Rocket adapter for Heisenberg
//!
//! This module provides helper functions for integrating Heisenberg
//! with Rocket applications.

use crate::core::config::Heisenberg;
use crate::core::mode::{detect_mode, Mode};
use rocket::response::{Responder, Response};
use rocket::Request;
use std::io::Cursor;
use std::path::Path;

/// Serve SPA content through Rocket
///
/// This function handles both development (proxy) and production (embedded assets) modes
/// automatically based on the current mode detection.
///
/// # Arguments
/// * `path` - The requested path
/// * `config` - Heisenberg configuration
///
/// # Returns
/// * `Ok(RocketResponse)` - The response with SPA content or proxied content
/// * `Err(rocket::http::Status)` - If serving fails
///
/// # Example
/// ```rust,no_run
/// use rocket::{get, routes, launch};
/// use std::path::PathBuf;
/// use heisenberg::{Heisenberg, adapters::rocket::{serve_spa, RocketResponse}};
///
/// #[get("/<path..>")]
/// async fn spa_handler(path: PathBuf) -> Result<RocketResponse, rocket::http::Status> {
///     let config = Heisenberg::new().spa("./dist").build();
///     serve_spa(&path, &config).await
/// }
///
/// #[get("/")]
/// fn api_handler() -> &'static str {
///     "API response"
/// }
///
/// #[launch]
/// fn rocket() -> _ {
///     rocket::build()
///         .mount("/api", routes![api_handler])
///         .mount("/", routes![spa_handler])
/// }
/// ```
pub async fn serve_spa(
    path: &Path,
    config: &Heisenberg,
) -> Result<RocketResponse, rocket::http::Status> {
    let path_str = path.to_string_lossy();
    let mode = detect_mode();

    println!(
        "🔍 Rocket: path={}, mode={:?}, env={:?}",
        path_str,
        mode,
        std::env::var("HEISENBERG_MODE")
    );

    // Find matching route configuration
    let route_config = config
        .routes
        .iter()
        .find(|route| path_matches(&route.pattern, &path_str))
        .ok_or(rocket::http::Status::NotFound)?;

    match mode {
        Mode::Proxy => proxy_request(path, route_config).await,
        Mode::Embed => serve_embedded_asset(path, route_config).await,
    }
}

/// Serve SPA content with full URI (including query parameters)
///
/// This is the preferred function for Rocket handlers as it preserves query strings.
pub async fn serve_spa_uri(
    uri: &str,
    config: &Heisenberg,
) -> Result<RocketResponse, rocket::http::Status> {
    let mode = detect_mode();

    println!("🔍 Rocket URI: uri={}, mode={:?}", uri, mode);

    // Extract path without query for route matching
    let path_only = uri.split('?').next().unwrap_or(uri);

    // Find matching route configuration
    let route_config = config
        .routes
        .iter()
        .find(|route| path_matches(&route.pattern, path_only))
        .ok_or(rocket::http::Status::NotFound)?;

    match mode {
        Mode::Proxy => proxy_request_uri(uri, route_config).await,
        Mode::Embed => serve_embedded_asset(Path::new(path_only), route_config).await,
    }
}

/// Custom response type for Rocket
pub struct RocketResponse {
    inner: Response<'static>,
}

impl<'r> Responder<'r, 'static> for RocketResponse {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        Ok(self.inner)
    }
}

/// Check if a path matches a route pattern
fn path_matches(pattern: &str, path: &str) -> bool {
    if pattern == "/*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("/*") {
        return path.starts_with(prefix);
    }
    pattern == path
}

/// Proxy request to development server
async fn proxy_request(
    path: &Path,
    route_config: &crate::core::config::SpaRouteConfig,
) -> Result<RocketResponse, rocket::http::Status> {
    let client = reqwest::Client::new();
    let path_str = path.to_string_lossy();
    let target_url = format!(
        "{}/{}",
        route_config.dev_proxy_url.trim_end_matches('/'),
        path_str.trim_start_matches('/')
    );

    println!("🌐 Proxying: {} -> {}", path_str, target_url);

    let response = client
        .get(&target_url)
        .send()
        .await
        .map_err(|_| rocket::http::Status::BadGateway)?;

    let status_code = response.status().as_u16();
    let headers = response.headers().clone();

    println!(
        "📦 Response: status={}, content-type={:?}",
        status_code,
        headers.get("content-type")
    );
    let body = response
        .bytes()
        .await
        .map_err(|_| rocket::http::Status::BadGateway)?;

    let rocket_status = rocket::http::Status::from_code(status_code)
        .unwrap_or(rocket::http::Status::InternalServerError);

    let mut response_builder = Response::build();
    response_builder.status(rocket_status);

    // Copy content-type and other important headers
    if let Some(content_type) = headers.get("content-type") {
        if let Ok(ct_str) = content_type.to_str() {
            println!("🔧 Parsing content-type: {}", ct_str);
            if let Some(content_type) = rocket::http::ContentType::parse_flexible(ct_str) {
                println!("✅ Setting content-type: {:?}", content_type);
                response_builder.header(content_type);
            } else {
                println!("❌ Failed to parse content-type");
            }
        }
    } else {
        println!("⚠️  No content-type header from Vite");
    }

    let response = response_builder
        .sized_body(body.len(), Cursor::new(body))
        .finalize();

    Ok(RocketResponse { inner: response })
}

/// Proxy request with full URI (including query parameters)
async fn proxy_request_uri(
    uri: &str,
    route_config: &crate::core::config::SpaRouteConfig,
) -> Result<RocketResponse, rocket::http::Status> {
    let client = reqwest::Client::new();
    let target_url = format!(
        "{}/{}",
        route_config.dev_proxy_url.trim_end_matches('/'),
        uri.trim_start_matches('/')
    );

    println!("🌐 Proxying URI: {} -> {}", uri, target_url);

    let response = client
        .get(&target_url)
        .send()
        .await
        .map_err(|_| rocket::http::Status::BadGateway)?;

    let status_code = response.status().as_u16();
    let headers = response.headers().clone();

    println!(
        "📦 Response: status={}, content-type={:?}",
        status_code,
        headers.get("content-type")
    );
    let body = response
        .bytes()
        .await
        .map_err(|_| rocket::http::Status::BadGateway)?;

    let rocket_status = rocket::http::Status::from_code(status_code)
        .unwrap_or(rocket::http::Status::InternalServerError);

    let mut response_builder = Response::build();
    response_builder.status(rocket_status);

    // Copy content-type header
    if let Some(content_type) = headers.get("content-type") {
        if let Ok(ct_str) = content_type.to_str() {
            if let Some(content_type) = rocket::http::ContentType::parse_flexible(ct_str) {
                response_builder.header(content_type);
            }
        }
    }

    let response = response_builder
        .sized_body(body.len(), Cursor::new(body))
        .finalize();

    Ok(RocketResponse { inner: response })
}

/// Serve embedded asset in production mode
async fn serve_embedded_asset(
    path: &Path,
    route_config: &crate::core::config::SpaRouteConfig,
) -> Result<RocketResponse, rocket::http::Status> {
    use tokio::fs;

    let path_str = path.to_string_lossy();
    let normalized_path = path_str.trim_start_matches('/');

    // Try to serve the requested file
    let file_path = route_config.embed_dir.join(normalized_path);
    if let Ok(content) = fs::read(&file_path).await {
        let mime_type = detect_mime_type_rocket(normalized_path);
        let content_type = rocket::http::ContentType::parse_flexible(mime_type)
            .unwrap_or(rocket::http::ContentType::Binary);

        let response = Response::build()
            .status(rocket::http::Status::Ok)
            .header(content_type)
            .sized_body(content.len(), Cursor::new(content))
            .finalize();

        return Ok(RocketResponse { inner: response });
    }

    // Fallback to index.html for SPA routing
    let index_path = route_config.embed_dir.join("index.html");
    if let Ok(content) = fs::read(&index_path).await {
        let response = Response::build()
            .status(rocket::http::Status::Ok)
            .header(rocket::http::ContentType::HTML)
            .sized_body(content.len(), Cursor::new(content))
            .finalize();

        return Ok(RocketResponse { inner: response });
    }

    Err(rocket::http::Status::NotFound)
}

fn detect_mime_type_rocket(path: &str) -> &'static str {
    let ext = std::path::Path::new(path)
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
