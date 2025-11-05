//! # Heisenberg
//!
//! Framework-agnostic dual-mode web serving for Rust applications.
//!
//! Heisenberg provides seamless switching between proxy mode (forwarding to frontend dev servers)
//! and embed mode (serving embedded static assets) without being opinionated about your web framework.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use axum::{routing::get, Router};
//! use heisenberg::{Heisenberg, HeisenbergLayer};
//!
//! #[tokio::main]
//! async fn main() {
//!     let spa = heisenberg::embed_spa!("./web/build");
//!     let config = Heisenberg::new()
//!         .route("/*", spa)
//!         .dev_server("http://localhost:5173")
//!         .build();
//!
//!     let app = Router::new()
//!         .route("/api/hello", get(|| async { "Hello API!" }))
//!         .layer(HeisenbergLayer::new(config));
//!
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```
//!
//! ## Features
//!
//! - **Framework Agnostic**: Works with any Tower-based framework (Axum, Warp, Hyper)
//! - **Dual Mode**: Automatic proxy/embed mode switching
//! - **Smart Inference**: Automatically detects frontend configuration from package.json
//! - **Process Management**: Handles frontend dev server lifecycle
//! - **SPA Support**: Client-side routing with fallback to index.html
//! - **Optional Logging**: Structured diagnostics with `tracing` (enable `logging` feature)
//!
//! ## Mode Detection
//!
//! - **Proxy Mode**: `cargo run` → Forward to frontend dev server
//! - **Embed Mode**: `cargo build --release` → Serve embedded assets
//! - **Override**: `HEISENBERG_MODE=embed|proxy` environment variable

#![warn(missing_docs)]

pub mod config;
pub mod core;
pub mod error;
pub mod services;
pub mod utils;

// Re-export the embed_spa macro
pub use heisenberg_macros::embed_spa;

#[cfg(feature = "axum")]
pub mod tower;

// Framework adapters
#[cfg(any(feature = "actix", feature = "rocket"))]
pub mod adapters;

// Re-export main types
pub use crate::core::config::{Heisenberg, SpaRouteBuilder};
pub use crate::core::embedded_spa::EmbeddedSpa;
pub use crate::error::HeisenbergError;
pub use crate::utils::shutdown_signal;

#[cfg(feature = "axum")]
pub use crate::tower::{HeisenbergLayer, HeisenbergService};

// Re-export dependencies needed by the embed_spa! macro
//
// These are re-exported so the macro-generated code can reference them as
// ::heisenberg::ctor, ::heisenberg::paste, etc. This ensures version compatibility.
//
// Note: Users still need rust-embed in their Cargo.toml because #[derive(RustEmbed)]
// requires the crate to be in the user's dependency graph (Rust derive macro limitation).
// However, the re-export ensures the macro uses the correct version even if the user
// accidentally specifies a different version.
//
// ctor and paste work through re-exports (attribute/proc macros), so users don't need them.
#[doc(hidden)]
pub use ctor;
#[doc(hidden)]
pub use paste;
#[doc(hidden)]
pub use rust_embed;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_config() {
        let spa = EmbeddedSpa::new("./dist", "");
        let config = Heisenberg::new().route("/*", spa).build();
        assert_eq!(config.routes.len(), 1);
        assert_eq!(config.routes[0].pattern, "/*");
    }
}
