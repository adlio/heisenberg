//! Tower service implementation

use crate::core::config::Heisenberg;
use crate::core::mode::{detect_mode, Mode};
use crate::core::router::Router;
use crate::services::process::ProcessManager;
use crate::services::proxy::ProxyService;
use http_body_util::BodyExt;
use hyper::body::Bytes;
use hyper::{Request, Response};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tower::Service;
#[cfg(feature = "logging")]
use tracing::debug;

/// Heisenberg Tower service
#[derive(Clone)]
pub struct HeisenbergService<S> {
    inner: S,
    router: Arc<Mutex<Router>>,
    mode: Mode,
    proxy_services: Arc<Mutex<HashMap<String, Arc<ProxyService>>>>,
    #[allow(dead_code)] // Kept alive for Drop cleanup
    process_manager: Option<Arc<ProcessManager>>,
}

impl<S> HeisenbergService<S> {
    /// Create a new Heisenberg service
    pub fn new(
        inner: S,
        config: Heisenberg,
        process_manager: Option<Arc<ProcessManager>>,
    ) -> Result<Self, crate::error::HeisenbergError> {
        let mode = detect_mode();
        let router = Router::new(config.routes().to_vec(), mode)?;

        // Pre-create proxy services for proxy mode
        let mut proxy_services = HashMap::new();

        if mode == Mode::Proxy {
            for route in config.routes() {
                let proxy = Arc::new(ProxyService::new(route.dev_proxy_url.clone()));
                proxy_services.insert(route.pattern.clone(), proxy);
            }
        }

        Ok(Self {
            inner,
            router: Arc::new(Mutex::new(router)),
            mode,
            proxy_services: Arc::new(Mutex::new(proxy_services)),
            process_manager,
        })
    }
}

impl<S, B> Service<Request<B>> for HeisenbergService<S>
where
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Response: Into<Response<B>> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + Sync + 'static,
    B: hyper::body::Body + From<Bytes> + Send + 'static,
    B::Data: Send,
    B::Error: std::error::Error + Send + Sync,
{
    type Response = Response<B>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let inner = self.inner.clone();
        let mut inner_service = inner;
        let router = self.router.clone();
        let mode = self.mode;
        let proxy_services = self.proxy_services.clone();

        Box::pin(async move {
            let path = req.uri().path().to_string();
            let query = req.uri().query().map(|s| s.to_string());
            let headers = req.headers().clone();

            #[cfg(feature = "logging")]
            debug!(path = %path, mode = ?mode, "Processing Heisenberg request");

            // Check if this is a WebSocket upgrade request
            let is_websocket = headers
                .get("upgrade")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.eq_ignore_ascii_case("websocket"))
                .unwrap_or(false);

            // Skip common API prefixes - let inner service handle them
            let is_api_path = path.starts_with("/api/") || path == "/api";

            // Try to match against Heisenberg routes (but skip API paths)
            let route_match = if !is_api_path {
                let mut router_guard = router.lock().unwrap();
                router_guard.match_route(&path).cloned()
            } else {
                None
            };

            if let Some(route_config) = route_match {
                #[cfg(feature = "logging")]
                debug!(pattern = %route_config.pattern, "Route matched");

                match mode {
                    Mode::Proxy => {
                        // Handle WebSocket upgrade requests specially
                        if is_websocket {
                            #[cfg(feature = "logging")]
                            debug!("WebSocket upgrade detected, proxying to dev server");

                            let proxy = {
                                let proxy_services = proxy_services.lock().unwrap();
                                proxy_services.get(&route_config.pattern).cloned()
                            };
                            if let Some(proxy) = proxy {
                                match proxy.proxy_websocket(req).await {
                                    Ok(response) => {
                                        let (parts, body) = response.into_parts();
                                        let bytes = body.collect().await.unwrap().to_bytes();
                                        return Ok(Response::from_parts(parts, B::from(bytes)));
                                    }
                                    Err(e) => {
                                        #[cfg(feature = "logging")]
                                        debug!(error = %e, "WebSocket proxy failed");
                                        return Ok(Response::builder()
                                            .status(503)
                                            .body(B::from(Bytes::from(format!(
                                                "WebSocket proxy error: {}",
                                                e
                                            ))))
                                            .unwrap());
                                    }
                                }
                            }
                        }

                        let proxy = {
                            let proxy_services = proxy_services.lock().unwrap();
                            proxy_services.get(&route_config.pattern).cloned()
                        };
                        if let Some(proxy) = proxy {
                            println!(
                                "→ {} {} → {}",
                                req.method(),
                                path,
                                route_config.dev_proxy_url
                            );
                            match proxy.proxy_request(&path, query.as_deref(), &headers).await {
                                Ok(response) => {
                                    let (parts, body) = response.into_parts();
                                    let bytes = body.collect().await.unwrap().to_bytes();
                                    return Ok(Response::from_parts(parts, B::from(bytes)));
                                }
                                Err(e) => {
                                    #[cfg(feature = "logging")]
                                    debug!(error = %e, "Proxy request failed");
                                    return Ok(Response::builder()
                                        .status(503)
                                        .body(B::from(Bytes::from(format!("Proxy error: {}", e))))
                                        .unwrap());
                                }
                            }
                        }
                    }
                    Mode::Embed => {
                        // Strip route prefix from path
                        let stripped_path = if route_config.pattern.ends_with("/*") {
                            let prefix = route_config.pattern.trim_end_matches("/*");
                            path.strip_prefix(prefix)
                                .unwrap_or(&path)
                                .trim_start_matches('/')
                        } else {
                            path.trim_start_matches('/')
                        };

                        // Serve from embedded assets, honoring the client's
                        // If-None-Match header for 304 revalidation.
                        let if_none_match = headers
                            .get("if-none-match")
                            .and_then(|v| v.to_str().ok())
                            .map(|s| s.to_string());
                        match crate::services::embed_registry::serve_embedded_asset_cached(
                            &route_config.embed_dir.to_string_lossy(),
                            stripped_path,
                            route_config.fallback_file.as_deref(),
                            if_none_match.as_deref(),
                        ) {
                            Ok(response) => {
                                let (parts, body) = response.into_parts();
                                let bytes = body.collect().await.unwrap().to_bytes();
                                return Ok(Response::from_parts(parts, B::from(bytes)));
                            }
                            Err(e) => {
                                #[cfg(feature = "logging")]
                                debug!(error = %e, "Embedded asset not found");
                                return Ok(Response::builder()
                                    .status(404)
                                    .body(B::from(Bytes::from(format!(
                                        "Asset not found (embed mode requires embed_spa! macro): {}",
                                        e
                                    ))))
                                    .unwrap());
                            }
                        }
                    }
                }
            }

            // No Heisenberg route matched, pass to inner service
            match inner_service.call(req).await {
                Ok(response) => Ok(response.into()),
                Err(e) => Err(e),
            }
        })
    }
}
