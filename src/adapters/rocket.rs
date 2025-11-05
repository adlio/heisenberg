//! Rocket adapter for Heisenberg
//!
//! This module provides helper functions for integrating Heisenberg
//! with Rocket applications.

use crate::core::config::Heisenberg;
use crate::core::mode::{detect_mode, Mode};
use http_body_util::BodyExt;
use rocket::response::{Responder, Response};
use rocket::{
    get,
    request::{self, FromRequest, Request},
    routes, Route, State,
};
use std::io::Cursor;
use std::path::Path;

/// Serve SPA content through Rocket with full URI support
///
/// This function handles both proxy and embed modes
/// automatically based on the current mode detection. Preserves query parameters for
/// Vite HMR and cache-busting.
///
/// # Arguments
/// * `uri` - The full URI string including query parameters
/// * `config` - Heisenberg configuration
///
/// # Returns
/// * `Ok(RocketResponse)` - The response with SPA content or proxied content
/// * `Err(rocket::http::Status)` - If serving fails
///
/// # Example
/// ```rust,no_run
/// use rocket::{get, routes, launch, request::{self, FromRequest, Request}};
/// use heisenberg::{Heisenberg, adapters::rocket::{serve_spa, RocketResponse}};
///
/// // Request guard to capture full URI
/// pub struct FullUri(String);
///
/// #[rocket::async_trait]
/// impl<'r> FromRequest<'r> for FullUri {
///     type Error = ();
///     async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, ()> {
///         let path = req.uri().path().as_str();
///         let full = if let Some(query) = req.uri().query() {
///             format!("{}?{}", path, query.as_str())
///         } else {
///             path.to_string()
///         };
///         request::Outcome::Success(FullUri(full))
///     }
/// }
///
/// #[get("/<_..>")]
/// async fn spa_handler(uri: FullUri, config: &rocket::State<Heisenberg>) -> Result<RocketResponse, rocket::http::Status> {
///     serve_spa(&uri.0, config).await
/// }
/// ```
pub async fn serve_spa(
    uri: &str,
    config: &Heisenberg,
) -> Result<RocketResponse, rocket::http::Status> {
    let mode = detect_mode();

    println!("🔍 Rocket: uri={}, mode={:?}", uri, mode);

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

    println!("→ GET {} → {}", uri, route_config.dev_proxy_url);

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

/// Serve embedded asset from embed registry
async fn serve_embedded_asset(
    path: &Path,
    route_config: &crate::core::config::SpaRouteConfig,
) -> Result<RocketResponse, rocket::http::Status> {
    let path_str = path.to_string_lossy();
    let stripped_path = path_str.trim_start_matches('/');

    match crate::services::embed_registry::serve_embedded_asset(
        &route_config.embed_dir.to_string_lossy(),
        stripped_path,
        route_config.fallback_file.as_deref(),
    ) {
        Ok(hyper_response) => {
            let status = hyper_response.status();
            let headers = hyper_response.headers().clone();
            let body = hyper_response
                .into_body()
                .collect()
                .await
                .map_err(|_| rocket::http::Status::InternalServerError)?
                .to_bytes();

            let mut response = Response::build();
            response.status(
                rocket::http::Status::from_code(status.as_u16())
                    .unwrap_or(rocket::http::Status::InternalServerError),
            );

            for (name, value) in headers.iter() {
                if let Ok(value_str) = value.to_str() {
                    response.raw_header(name.to_string(), value_str.to_string());
                }
            }

            let response = response
                .sized_body(body.len(), Cursor::new(body.to_vec()))
                .finalize();

            Ok(RocketResponse { inner: response })
        }
        Err(_) => Err(rocket::http::Status::NotFound),
    }
}

/// Request guard to capture full URI with query string
pub struct FullUri(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for FullUri {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let path = req.uri().path().as_str();
        let full = if let Some(query) = req.uri().query() {
            format!("{}?{}", path, query.as_str())
        } else {
            path.to_string()
        };
        request::Outcome::Success(FullUri(full))
    }
}

/// Pre-built route handler for SPA root (/)
#[get("/", rank = 1)]
pub async fn spa_root(config: &State<Heisenberg>) -> Result<RocketResponse, rocket::http::Status> {
    serve_spa("index.html", config).await
}

/// Pre-built route handler for SPA catchall (/<_..>)
#[get("/<_..>", rank = 2)]
pub async fn spa_catchall(
    uri: FullUri,
    config: &State<Heisenberg>,
) -> Result<RocketResponse, rocket::http::Status> {
    serve_spa(&uri.0, config).await
}

/// Generate routes for SPA serving
///
/// Returns a Vec of routes that can be mounted to serve a SPA.
///
/// # Example
/// ```rust,no_run
/// use heisenberg::{Heisenberg, adapters::rocket::spa_routes};
/// use rocket::launch;
///
/// #[launch]
/// fn rocket() -> _ {
///     let config = Heisenberg::from_working_dir("./web").build();
///     
///     rocket::build()
///         .manage(config)
///         .mount("/", spa_routes())
/// }
/// ```
pub fn spa_routes() -> Vec<Route> {
    routes![spa_root, spa_catchall]
}
