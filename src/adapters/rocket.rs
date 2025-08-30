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

    // Find matching route configuration
    let route_config = config
        .routes
        .iter()
        .find(|route| path_matches(&route.pattern, &path_str))
        .ok_or(rocket::http::Status::NotFound)?;

    match mode {
        Mode::Development => proxy_request(path, route_config).await,
        Mode::Production => serve_embedded_asset(path, route_config).await,
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

    let response = client
        .get(&target_url)
        .send()
        .await
        .map_err(|_| rocket::http::Status::BadGateway)?;

    let status_code = response.status().as_u16();
    let body = response
        .bytes()
        .await
        .map_err(|_| rocket::http::Status::BadGateway)?;

    let rocket_status = rocket::http::Status::from_code(status_code)
        .unwrap_or(rocket::http::Status::InternalServerError);

    let response = Response::build()
        .status(rocket_status)
        .sized_body(body.len(), Cursor::new(body))
        .finalize();

    Ok(RocketResponse { inner: response })
}

/// Serve embedded asset in production mode
async fn serve_embedded_asset(
    path: &Path,
    route_config: &crate::core::config::SpaRouteConfig,
) -> Result<RocketResponse, rocket::http::Status> {
    // For now, return a placeholder response
    // This will be implemented when we have the static file service ready
    let path_str = path.to_string_lossy();
    let html_content = format!(
        "<html><body><h1>Production Mode</h1><p>Path: {}</p><p>Route: {}</p></body></html>",
        path_str, route_config.pattern
    );

    let response = Response::build()
        .status(rocket::http::Status::Ok)
        .header(rocket::http::ContentType::HTML)
        .sized_body(html_content.len(), Cursor::new(html_content))
        .finalize();

    Ok(RocketResponse { inner: response })
}
