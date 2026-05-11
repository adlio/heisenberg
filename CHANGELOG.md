# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Smart HTTP caching for embedded assets**: embed mode now emits strong
  content-hash `ETag` headers, returns `304 Not Modified` on matching
  `If-None-Match`, and picks a `Cache-Control` policy from the asset path:
  - Fingerprinted assets (e.g. `app.abc12345.js`, files inside
    `/assets/`, `/_app/immutable/`, `/_next/static/`, `/_astro/`) get
    `public, max-age=31536000, immutable`.
  - HTML and bare SPA routes get `no-cache` so deployments are picked up
    immediately without a hard refresh.
  - Everything else gets `public, max-age=3600, must-revalidate`.
- New public API `services::embed_registry::serve_embedded_asset_cached`
  and `services::cache` module (`CachePolicy`, `policy_for_path`,
  `compute_etag`, `etag_for`, `if_none_match`).
- Rocket adapter: new `serve_spa_with_request` helper that threads the
  client's `If-None-Match` header through to the cache layer.

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
