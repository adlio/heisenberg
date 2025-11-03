# Heisenberg

[![Crates.io](https://img.shields.io/crates/v/heisenberg.svg)](https://crates.io/crates/heisenberg)
[![Documentation](https://docs.rs/heisenberg/badge.svg)](https://docs.rs/heisenberg)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

Framework-agnostic dual-mode web serving for Rust applications. Seamlessly switch between proxy mode (forwarding to frontend dev servers) and embed mode (serving embedded static assets).

## ✨ Features

- **🔄 Dual Mode**: Automatic proxy/embed mode switching
- **🎯 Framework Agnostic**: Works with Axum, Warp, Actix-web, Rocket, and more
- **🧠 Smart Inference**: Auto-detects frontend configuration from package.json
- **⚡ Zero Config**: Works out-of-the-box with sensible defaults
- **🔧 Process Management**: Handles frontend dev server lifecycle
- **🔌 WebSocket Proxying**: Transparent HMR support for Vite, Next.js, CRA
- **📱 SPA Support**: Client-side routing with fallback to index.html
- **📊 Optional Logging**: Structured diagnostics with `tracing`

## 🚀 Quick Start

### 1. Add to your Cargo.toml

```toml
[dependencies]
heisenberg = "0.2"
axum = "0.7"
tokio = { version = "1.35", features = ["full"] }
```

### 2. Basic setup

```rust
use axum::{routing::get, Router};
use heisenberg::{Heisenberg, HeisenbergLayer};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/hello", get(|| async { "Hello API!" }))
        .layer(HeisenbergLayer::new(Heisenberg::new().spa("./dist").build()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    
    // Graceful shutdown cleans up dev servers on Ctrl+C
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
```

### 3. Run in different modes

```bash
# Proxy mode - forwards to frontend dev server
cargo run

# Embed mode - serves pre-built assets from disk
# (Build frontend first: cd frontend && npm run build)
cargo build --release && ./target/release/your-app
```

That's it! Heisenberg automatically:
- 🔍 Finds your `package.json` and extracts the dev command
- 🚀 Starts your frontend dev server (`npm run dev`) in proxy mode
- 🔗 Proxies frontend requests (including WebSocket HMR)
- 📁 Serves pre-built assets from disk in embed mode
- 🌐 Opens your browser automatically

## 📖 Documentation

- **[API Documentation](https://docs.rs/heisenberg)** - Complete API reference
- **[Examples](examples/)** - Working examples for different frameworks

## 🎯 Framework Support

### Tower-based (Zero Config)
Works automatically with any Tower-based framework:

```rust
// Axum
let app = Router::new()
    .route("/api/hello", get(handler))
    .layer(HeisenbergLayer::new(heisenberg_config));
```

### Framework Adapters
Helper functions for non-Tower frameworks:

```rust
// Actix-web
use heisenberg::actix::serve_spa;

// Rocket
use heisenberg::rocket::serve_spa;
```

## ⚙️ Configuration

### Smart Defaults
```rust
// Infers everything from your project structure
Heisenberg::new().spa("./dist").build()
```

### Advanced Configuration
```rust
Heisenberg::new()
    .spa("./frontend/dist")                   // Must match build output directory
        .dev_server("http://localhost:3000")  // Override auto-detected port
        .dev_command(["npm", "run", "dev"])   // Override auto-detected command
        .open_browser(true)
    .build()
```

**Port Detection:** Heisenberg automatically detects dev server ports from:
- CLI flags in package.json scripts (`--port 3000`, `-p 5173`)
- Literal port numbers in vite.config.js (`port: 5173`)
- Framework defaults (Vite→5173, Next.js→3000, CRA→3000)

**Note:** Dynamic port configuration (variables, expressions) requires manual override with `.dev_server()`.

### Custom Build Commands
If you use a custom build command (not `npm run build`):

```bash
# Run your custom build command first
cd frontend && npm run prod && cd ..

# Then build Rust binary
cargo build --release
```

The `.spa("./path")` must point to wherever your build outputs files.

### Dynamic Build Directories
For dynamic output paths, use environment variables:

```rust
let build_dir = std::env::var("FRONTEND_BUILD_DIR")
    .unwrap_or_else(|_| "./frontend/dist".to_string());
let config = Heisenberg::new().spa(&build_dir).build();
```

### Multiple SPAs
```rust
Heisenberg::new()
    .spa("./admin/dist")
        .dev_server("http://localhost:3001")
    .spa("./app/dist")
        .dev_server("http://localhost:3000")
    .build()
```

## 🔧 Mode Detection

| Build Command | Mode | Behavior |
|---------------|------|----------|
| `cargo run` | Proxy | Forward to dev server |
| `cargo build --release` | Embed | Serve pre-built assets from disk |
| `HEISENBERG_MODE=embed cargo run` | Embed | Force embed mode |
| `HEISENBERG_MODE=proxy cargo build --release` | Proxy | Force proxy mode |

**Important:** Embed mode serves files from the directory specified in `.spa("./path")`. You must build your frontend assets first (e.g., `npm run build`) before running in embed mode.

## 📊 Debugging

Enable structured logging:

```toml
[dependencies]
heisenberg = { version = "0.2", features = ["logging"] }
tracing-subscriber = "0.3"
```

```bash
RUST_LOG=debug,heisenberg=trace cargo run
```

## 🏗️ Examples

- **[SvelteKit](examples/axum-sveltekit/)** - ⭐ Showcase example with WebSocket HMR
- **[Basic Axum](examples/axum-simple/)** - Simple Axum + React setup
- **[Multi-SPA](examples/axum-multi-spa/)** - Multiple frontend applications
- **[Actix-web](examples/actix-react/)** - Actix-web integration
- **[Rocket](examples/rocket-vue/)** - Rocket integration
- **[Logging](examples/logging-example/)** - Structured logging example

## 🧪 Testing

```bash
# Run all tests including WebSocket proxying
cargo test

# Run specific WebSocket test
cargo test --test websocket_proxy

# Try the showcase example
cd examples/axum-sveltekit && cargo run
```

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.