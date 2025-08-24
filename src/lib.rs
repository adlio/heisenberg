//! # Heisenberg
//!
//! Framework-agnostic dual-mode web serving for Rust applications.
//!
//! Heisenberg provides seamless switching between development mode (proxying to frontend dev servers)
//! and production mode (serving embedded static assets) without being opinionated about your web framework.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use axum::{routing::get, Router};
//! use heisenberg::Heisenberg;
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Router::new()
//!         .route("/api/hello", get(|| async { "Hello API!" }))
//!         .layer(Heisenberg::new().spa("./dist"));
//!
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```
//!
//! ## Features
//!
//! - **Framework Agnostic**: Works with any Tower-based framework (Axum, Warp, Hyper)
//! - **Dual Mode**: Automatic dev/prod mode switching
//! - **Smart Inference**: Automatically detects frontend configuration from package.json
//! - **Process Management**: Handles frontend dev server lifecycle
//! - **SPA Support**: Client-side routing with fallback to index.html
//! - **Optional Logging**: Structured diagnostics with `tracing` (enable `logging` feature)
//!
//! ## Mode Detection
//!
//! - **Development**: `cargo run` → Proxy to frontend dev server
//! - **Production**: `cargo build --release` → Serve embedded assets
//! - **Override**: `HEISENBERG_MODE=embed|proxy` environment variable

#![warn(missing_docs)]

pub mod core;
pub mod error;
pub mod services;
pub mod utils;

#[cfg(feature = "tower")]
pub mod tower;

// Framework adapters
#[cfg(any(feature = "actix", feature = "rocket"))]
pub mod adapters;

// Re-export main types
pub use crate::core::config::{Heisenberg, SpaRouteBuilder};
pub use crate::error::HeisenbergError;

#[cfg(feature = "tower")]
pub use crate::tower::{HeisenbergLayer, HeisenbergService};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_config() {
        let config = Heisenberg::new().spa("./dist").build();
        assert_eq!(config.routes.len(), 1);
        assert_eq!(config.routes[0].pattern, "/*");
    }
}
