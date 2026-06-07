# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-06-06

### Changed

- **`reqwest` now uses `rustls-tls` instead of `native-tls`**: drops the dependency on the system OpenSSL/`libssl` (no more dynamic linking to the host's TLS stack). This makes binaries that depend on `heisenberg` more portable across Linux distributions and removes a build-time requirement for OpenSSL development headers.

### Internal

- Significant test coverage improvements across `core`, `services`, `embed_registry`, the `embed_spa!` macro, and the actix + rocket adapters.
- CI: example builds verified in GitHub Actions; coverage uploaded to Codecov via `cargo-llvm-cov`; pre-commit hook (`cargo-husky`) running `make ci` locally.
- Tag-triggered release workflow added: pushing a `vX.Y.Z` tag now creates a GitHub Release from the matching changelog section and publishes `heisenberg-macros`, `heisenberg`, and `cargo-heisenberg` to crates.io in dependency order.

## [0.4.0] - 2025-11-05

### Added

- **cargo-heisenberg CLI**: New cargo plugin for build orchestration
  - `cargo heisenberg init` - Generate heisenberg.toml with inferred defaults
  - `cargo heisenberg build` - Build frontend assets then run cargo build
  - `cargo heisenberg run` - Start frontend + backend with split-pane TUI
  - `--no-tui` flag for plain output mode
  - Auto-detection of frontends in `./web` or `./frontend` directories
  - Automatic `npm install` when node_modules is missing or stale

- **heisenberg.toml configuration file**: Optional config for multi-SPA setups
  - Single SPA syntax: `[spa]`
  - Multiple SPA syntax: `[[spa]]`
  - Fields: name, working_dir, output_dir, dev_command, build_command, dev_server

- **heisenberg-macros crate**: Separated `embed_spa!` macro into its own crate
  - Re-exported from main heisenberg crate for backward compatibility

- **Rocket adapter improvements**: Full query parameter support
  - `serve_spa()` now accepts full URI strings including query parameters
  - Proper MIME type detection and Content-Type headers

- **New example**: rocket-multi-spa with Vue frontends

- **Graceful shutdown helper**: `heisenberg::shutdown_signal()` for clean process termination

### Changed

- Examples now include actual frontend source code instead of pre-built dist folders
- Updated all examples to use `cargo heisenberg run` for development
- Rocket adapter `serve_spa()` signature changed from `&Path` to `&str` for URI handling

### Removed

- `axum-simple` example (superseded by axum-sveltekit)
- `logging-example` (logging is now standard)
- `websocket-demo` (WebSocket support documented in other examples)

## [0.3.0] - 2025-11-03

### Added

- **Compile-time asset embedding**: `embed_spa!()` macro embeds assets into the binary at compile time using rust-embed
- Global registry pattern for multiple SPAs
- Support for multiple SPAs with unique identifiers

### Changed

- **Breaking**: Assets now embedded in binary (previously served from disk)
- **Breaking**: Requires `rust-embed = "8.0"` in user's Cargo.toml

## [0.2.0] - 2025-11-02

### Added

- **WebSocket proxying**: Transparent proxying of WebSocket connections including HMR
- **Port auto-detection**: Reads port from vite.config.js with framework-specific fallbacks
- **Blocking dev server startup**: Dev servers start synchronously before Rust server binds

### Changed

- Consistent use of "proxy/embed" terminology instead of "development/production"

### Fixed

- Dev server startup race conditions
- Port conflict detection

## [0.1.1] - 2025-08-25

### Changed

- Bug fixes and code improvements

## [0.1.0] - 2025-08-25

### Added

- Core library with dual-mode web serving (proxy and embed)
- Tower layer and service implementation
- Framework adapters for Actix-web and Rocket
- Fluent builder API with package.json inference
- Automatic mode detection based on build profile
- Frontend dev server lifecycle management
- SPA routing with fallback to index.html
- Multi-SPA support

### Framework Support

- Axum (native Tower integration)
- Warp (native Tower integration)
- Actix-web (adapter)
- Rocket (adapter)
