//! Tower service implementation

use crate::core::config::Heisenberg;
use crate::core::mode::detect_mode;
use crate::core::router::Router;
use crate::tower::future::HeisenbergFuture;
use hyper::{Request, Response, StatusCode};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tower::Service;
#[cfg(feature = "logging")]
use tracing::debug;

/// Heisenberg Tower service
#[derive(Debug, Clone)]
pub struct HeisenbergService<S> {
    inner: S,
    router: Arc<Mutex<Router>>,
}

impl<S> HeisenbergService<S> {
    /// Create a new Heisenberg service
    pub fn new(inner: S, config: Heisenberg) -> Result<Self, crate::error::HeisenbergError> {
        let mode = detect_mode();
        let router = Router::new(config.routes().to_vec(), mode)?;

        Ok(Self {
            inner,
            router: Arc::new(Mutex::new(router)),
        })
    }
}

impl<S, B> Service<Request<B>> for HeisenbergService<S>
where
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Response: Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + Sync + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        HeisenbergFuture<Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let inner = self.inner.clone();
        let mut inner_service = inner;
        let router = self.router.clone();

        HeisenbergFuture::new(Box::pin(async move {
            let path = req.uri().path();

            #[cfg(feature = "logging")]
            debug!(path = %path, "Processing Heisenberg request");

            // Try to match against Heisenberg routes
            let route_match = {
                let mut router_guard = router.lock().unwrap();
                router_guard.match_route(path).cloned()
            };

            if let Some(_route_config) = route_match {
                // TODO: Handle SPA routing (proxy/static files) in next phase
                // For now, return a placeholder response
                let _response = Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "text/html")
                    .body("Heisenberg SPA route matched - implementation coming soon!")
                    .unwrap();

                // This is a type conversion issue - we need to handle it properly
                // For now, fall through to inner service
                inner_service.call(req).await
            } else {
                // No Heisenberg route matched, pass to inner service
                inner_service.call(req).await
            }
        }))
    }
}
