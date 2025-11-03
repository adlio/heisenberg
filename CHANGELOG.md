# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-08-25

### Changed
- Updated code improvements and bug fixes

## [0.1.0] - 2025-08-25

### Added
- **Core Library**: Framework-agnostic dual-mode web serving
- **Tower Integration**: Native Tower layer and service implementation
- **Framework Adapters**: Helper functions for Actix-web and Rocket
- **Smart Configuration**: Fluent builder API with package.json inference
- **Mode Detection**: Automatic proxy/embed mode switching based on build profile
- **Process Management**: Automatic frontend dev server lifecycle management
- **Asset Embedding**: Embed-mode static asset serving with rust-embed
- **SPA Support**: Client-side routing with fallback to index.html
- **Health Checking**: Out-of-band monitoring of frontend dev servers
- **Browser Opening**: Automatic browser launch in proxy mode
- **Multi-SPA Support**: Multiple frontend applications with different routes
- **Structured Logging**: Optional tracing integration for diagnostics

### Framework Support
- **Axum**: Native Tower integration (zero configuration)
- **Warp**: Native Tower integration (zero configuration)  
- **Actix-web**: Helper function adapter
- **Rocket**: Helper function adapter
- **Any Tower-based framework**: Works automatically

### Features
- **Zero Configuration**: Works out-of-the-box with sensible defaults
- **Smart Inference**: Auto-detects frontend configuration from package.json
- **Cross-Platform**: Windows, macOS, and Linux support
- **Performance Optimized**: Minimal overhead in both dev and prod modes
- **Security Hardened**: Path traversal prevention and input validation
- **Comprehensive Testing**: 47 tests covering all major functionality

### Examples
- **axum-simple**: Basic Axum + HTML setup
- **axum-sveltekit**: Full-featured SvelteKit integration
- **axum-multi-spa**: Multiple frontend applications
- **actix-react**: Actix-web + React integration
- **rocket-vue**: Rocket + Vue integration
- **logging-example**: Structured logging demonstration

### Documentation
- **User Guide**: Comprehensive setup and configuration guide
- **API Documentation**: Complete rustdoc coverage
- **Integration Examples**: Working examples for all supported frameworks
- **Performance Benchmarks**: Baseline performance measurements

[0.1.0]: https://github.com/username/heisenberg/releases/tag/v0.1.0
## [Unreleased] - WebSocket Proxying Feature

### Added
- **WebSocket proxying** for transparent HMR support (Vite, Next.js, CRA)
  - Automatic detection of `Upgrade: websocket` header
  - Bidirectional message forwarding between client and backend
  - Proper WebSocket handshake with `Sec-WebSocket-Accept`
  - Implementation in `src/services/proxy.rs` and `src/tower/service.rs`

- **Automated testing** for WebSocket functionality
  - New test: `tests/websocket_proxy.rs`
  - Verifies end-to-end WebSocket proxying
  - Tests bidirectional communication
  - Run with: `cargo test --test websocket_proxy`

- **Enhanced axum-sveltekit example** as showcase
  - One-command experience: `cargo run`
  - Automatic Vite dev server startup
  - WebSocket HMR working out of the box
  - Comprehensive documentation in example directory

### Changed
- Updated README.md to highlight WebSocket support
- Simplified example configuration (removed manual working_dir)
- Added `axum` to dev-dependencies for testing

### Documentation
- All WebSocket documentation consolidated in example READMEs
- Root README.md updated with testing instructions
- Removed temporary documentation files (WEBSOCKET_*.md, TESTING*.md)
