//! Actix-web adapter for Heisenberg
//!
//! This module provides helper functions for integrating Heisenberg
//! with Actix-web applications.

use crate::core::config::Heisenberg;
use crate::core::mode::{detect_mode, Mode};
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};

/// Serve SPA content through Actix-web
///
/// This function handles both development (proxy) and production (embedded assets) modes
/// automatically based on the current mode detection.
///
/// # Arguments
/// * `req` - The Actix-web HTTP request
/// * `config` - Heisenberg configuration
///
/// # Returns
/// * `Ok(HttpResponse)` - The response with SPA content or proxied content
/// * `Err(actix_web::Error)` - If serving fails
///
/// # Example
/// ```rust,no_run
/// use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};
/// use heisenberg::{Heisenberg, adapters::actix::serve_spa};
///
/// async fn spa_handler(req: HttpRequest) -> Result<HttpResponse> {
///     let config = Heisenberg::new().spa("./dist").build();
///     serve_spa(&req, &config).await
/// }
///
/// async fn api_handler() -> Result<HttpResponse> {
///     Ok(HttpResponse::Ok().json("API response"))
/// }
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     HttpServer::new(|| {
///         App::new()
///             .route("/api/*", web::get().to(api_handler))
///             .route("/*", web::get().to(spa_handler))
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
pub async fn serve_spa(req: &HttpRequest, config: &Heisenberg) -> ActixResult<HttpResponse> {
    let path = req.path();
    let mode = detect_mode();

    // Find matching route configuration
    let route_config = config
        .routes
        .iter()
        .find(|route| path_matches(&route.pattern, path))
        .ok_or_else(|| actix_web::error::ErrorNotFound("No matching SPA route found"))?;

    match mode {
        Mode::Development => proxy_request(req, route_config).await,
        Mode::Production => serve_embedded_asset(path, route_config).await,
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
    req: &HttpRequest,
    route_config: &crate::core::config::SpaRouteConfig,
) -> ActixResult<HttpResponse> {
    let client = reqwest::Client::new();
    let target_url = format!("{}{}", route_config.dev_proxy_url, req.path());

    let response = client
        .get(&target_url)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorBadGateway(format!("Proxy error: {}", e)))?;

    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .bytes()
        .await
        .map_err(|e| actix_web::error::ErrorBadGateway(format!("Proxy body error: {}", e)))?;

    let mut actix_response = HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status.as_u16())
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    );

    // Copy relevant headers
    for (name, value) in headers.iter() {
        if let Ok(header_name) =
            actix_web::http::header::HeaderName::from_bytes(name.as_str().as_bytes())
        {
            if let Ok(header_value) =
                actix_web::http::header::HeaderValue::from_bytes(value.as_bytes())
            {
                actix_response.insert_header((header_name, header_value));
            }
        }
    }

    Ok(actix_response.body(body))
}

/// Serve embedded asset in production mode
async fn serve_embedded_asset(
    path: &str,
    route_config: &crate::core::config::SpaRouteConfig,
) -> ActixResult<HttpResponse> {
    // Normalize the path - remove leading slash and handle root
    let file_path = if path == "/" || path.is_empty() {
        route_config.fallback_file.as_deref().unwrap_or("index.html")
    } else {
        path.strip_prefix('/').unwrap_or(path)
    };
    
    // Build full file path
    let full_path = route_config.embed_dir.join(file_path);
    
    // Try to read the file
    match tokio::fs::read(&full_path).await {
        Ok(contents) => {
            // Determine content type from file extension
            let content_type = match full_path.extension().and_then(|ext| ext.to_str()) {
                Some("html") => "text/html; charset=utf-8",
                Some("css") => "text/css; charset=utf-8", 
                Some("js") => "application/javascript; charset=utf-8",
                Some("json") => "application/json; charset=utf-8",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("gif") => "image/gif",
                Some("svg") => "image/svg+xml",
                Some("ico") => "image/x-icon",
                Some("woff") => "font/woff",
                Some("woff2") => "font/woff2",
                Some("ttf") => "font/ttf",
                _ => "application/octet-stream",
            };
            
            Ok(HttpResponse::Ok()
                .content_type(content_type)
                .body(contents))
        }
        Err(_) => {
            // File not found, try fallback for SPA routing
            if let Some(fallback) = &route_config.fallback_file {
                let fallback_path = route_config.embed_dir.join(fallback);
                match tokio::fs::read(&fallback_path).await {
                    Ok(contents) => Ok(HttpResponse::Ok()
                        .content_type("text/html; charset=utf-8")
                        .body(contents)),
                    Err(_) => Err(actix_web::error::ErrorNotFound("File not found"))
                }
            } else {
                Err(actix_web::error::ErrorNotFound("File not found"))
            }
        }
    }
}
