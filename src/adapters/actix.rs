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
    if pattern.ends_with("/*") {
        let prefix = &pattern[..pattern.len() - 2];
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
    // For now, return a placeholder response
    // This will be implemented when we have the static file service ready
    Ok(HttpResponse::Ok().content_type("text/html").body(format!(
        "<html><body><h1>Production Mode</h1><p>Path: {}</p><p>Route: {}</p></body></html>",
        path, route_config.pattern
    )))
}
