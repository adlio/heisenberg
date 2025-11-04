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
rust-embed = "8.0"  # Required by embed_spa! macro
ctor = "0.2"        # Required by embed_spa! macro
paste = "1.0"       # Required by embed_spa! macro
```

**Note:** These dependencies are required because `embed_spa!()` uses proc macros that must run in your crate's compilation context.

### 2. Basic setup

```rust
use axum::{routing::get, Router};
use heisenberg::HeisenbergLayer;

#[tokio::main]
async fn main() {
    // Embed assets and configure
    let app = heisenberg::embed_spa!("./dist");
    let config = Heisenberg::new().route("/*", app).build();
    
    let router = Router::new()
        .route("/api/hello", get(|| async { "Hello API!" }))
        .layer(HeisenbergLayer::new(config));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    
    // Graceful shutdown cleans up dev servers on Ctrl+C
    axum::serve(listener, router)
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

# Embed mode - assets embedded in binary
# (Build frontend first: cd frontend && npm run build)
cargo build --release && ./target/release/your-app
```

That's it! Heisenberg automatically:
- 🔍 Finds your `package.json` and extracts the dev command
- 🚀 Starts your frontend dev server (`npm run dev`) in proxy mode
- 🔗 Proxies frontend requests (including WebSocket HMR)
- 📦 Embeds assets into binary in release builds
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
For custom dev server settings:

```rust
use heisenberg::Heisenberg;

// Embed assets (required for production)
heisenberg::embed_spa!("./frontend/dist");

// Configure with custom settings
let config = Heisenberg::new()
    .spa("./frontend/dist")
        .dev_server("http://localhost:3000")  // Override auto-detected port
        .dev_command(["npm", "run", "dev"])   // Override auto-detected command
        .open_browser(true)
    .build();
```

**Note:** Advanced configuration requires specifying the path twice - once for embedding, once for configuration.

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

# Then build Rust binary (assets embedded during compilation)
cargo build --release
```

Both `embed_spa!()` and `.spa()` must point to wherever your build outputs files.

### Multiple SPAs
```rust
// Embed each SPA with unique identifiers
let admin = heisenberg::embed_spa!("./admin/dist", admin);
let app = heisenberg::embed_spa!("./app/dist", app);

// Configure with route patterns
let config = Heisenberg::new()
    .route("/admin/*", admin)
    .route("/*", app)
    .build();
```



## 🔧 Mode Detection

| Build Command | Mode | Behavior |
|---------------|------|----------|
| `cargo run` | Proxy | Forward to dev server |
| `cargo build --release` | Embed | Serve embedded assets from binary |
| `HEISENBERG_MODE=embed cargo run` | Embed | Force embed mode |
| `HEISENBERG_MODE=proxy cargo build --release` | Proxy | Force proxy mode |

**Important:** Assets are embedded at compile time using `embed_spa!()`. You must build your frontend first (e.g., `npm run build`) before `cargo build --release`.

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