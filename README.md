# Heisenberg

[![Crates.io](https://img.shields.io/crates/v/heisenberg.svg)](https://crates.io/crates/heisenberg)
[![Documentation](https://docs.rs/heisenberg/badge.svg)](https://docs.rs/heisenberg)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

Framework-agnostic dual-mode web serving for Rust applications. Seamlessly switch between development mode (proxying to frontend dev servers) and production mode (serving embedded static assets).

## ✨ Features

- **🔄 Dual Mode**: Automatic dev/prod mode switching
- **🎯 Framework Agnostic**: Works with Axum, Warp, Actix-web, Rocket, and more
- **🧠 Smart Inference**: Auto-detects frontend configuration from package.json
- **⚡ Zero Config**: Works out-of-the-box with sensible defaults
- **🔧 Process Management**: Handles frontend dev server lifecycle
- **📱 SPA Support**: Client-side routing with fallback to index.html
- **📊 Optional Logging**: Structured diagnostics with `tracing`

## 🚀 Quick Start

### 1. Add to your Cargo.toml

```toml
[dependencies]
heisenberg = "0.1"
axum = "0.7"
tokio = { version = "1.35", features = ["full"] }
```

### 2. Basic setup

```rust
use axum::{routing::get, Router};
use heisenberg::Heisenberg;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/hello", get(|| async { "Hello API!" }))
        .layer(Heisenberg::new().spa("./dist"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### 3. Run in different modes

```bash
# Development mode - proxies to frontend dev server
cargo run

# Production mode - serves embedded assets  
cargo build --release && ./target/release/your-app
```

That's it! Heisenberg automatically:
- 🔍 Finds your `package.json` and extracts the dev command
- 🚀 Starts your frontend dev server (`npm run dev`)
- 🔗 Proxies frontend requests in development
- 📦 Embeds assets for production builds
- 🌐 Opens your browser automatically

## 📖 Documentation

- **[User Guide](GUIDE.md)** - Comprehensive setup and configuration guide
- **[API Documentation](https://docs.rs/heisenberg)** - Complete API reference
- **[Examples](examples/)** - Working examples for different frameworks

## 🎯 Framework Support

### Tower-based (Zero Config)
Works automatically with any Tower-based framework:

```rust
// Axum
let app = Router::new().layer(heisenberg_config);

// Warp  
let routes = routes.with(heisenberg_config);
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
Heisenberg::new().spa("./dist")
```

### Advanced Configuration
```rust
Heisenberg::new()
    .spa("./frontend/dist")
        .dev_server("http://localhost:3000")
        .dev_command(["npm", "run", "dev"])
        .open_browser(true)
    .build()
```

### Multiple SPAs
```rust
Heisenberg::new()
    .spa("/admin/*", "./admin/dist")
        .dev_server("http://localhost:3001")
    .spa("/*", "./app/dist")
        .dev_server("http://localhost:3000")
    .build()
```

## 🔧 Mode Detection

| Build Command | Mode | Behavior |
|---------------|------|----------|
| `cargo run` | Development | Proxy to dev server |
| `cargo build --release` | Production | Embed assets |
| `HEISENBERG_MODE=embed cargo run` | Production | Force embed mode |
| `HEISENBERG_MODE=proxy cargo build --release` | Development | Force proxy mode |

## 📊 Debugging

Enable structured logging:

```toml
[dependencies]
heisenberg = { version = "0.1", features = ["logging"] }
tracing-subscriber = "0.3"
```

```bash
RUST_LOG=debug,heisenberg=trace cargo run
```

## 🏗️ Examples

- **[Basic Axum](examples/axum-simple/)** - Simple Axum + React setup
- **[Logging](examples/logging-example/)** - Structured logging example
- **[Multi-SPA](examples/multi-spa/)** - Multiple frontend applications
- **[Actix-web](examples/actix-react/)** - Actix-web integration
- **[Rocket](examples/rocket-vue/)** - Rocket integration

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.