# Heisenberg

[![Crates.io](https://img.shields.io/crates/v/heisenberg.svg)](https://crates.io/crates/heisenberg)
[![Documentation](https://docs.rs/heisenberg/badge.svg)](https://docs.rs/heisenberg)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

The most developer-friendly way to serve SPAs in Rust. One line of code. Zero configuration. Automatic dev/prod mode switching.

## ✨ Features

- **⚡ One-Line Integration**: `.spa()` - that's it!
- **🔄 Dual Mode**: Automatic proxy (dev) / embed (prod) switching
- **🎯 Framework Agnostic**: Works with Axum, Actix-web, Rocket, and more
- **🧠 Smart Inference**: Auto-detects everything from your project structure
- **🛠️ Cargo Plugin**: Build orchestration with beautiful TUI
- **🔌 WebSocket Proxying**: Transparent HMR support for Vite, Next.js, etc.
- **📱 SPA Support**: Client-side routing with fallback to index.html
- **🌐 Multi-SPA**: Multiple frontends in one app

## 🚀 Quick Start

### 1. Add to your Cargo.toml

```toml
[dependencies]
heisenberg = "0.3"
axum = "0.7"
tokio = { version = "1.35", features = ["full"] }
```

### 2. One line of code

```rust
use axum::{routing::get, Router};
use heisenberg::SpaExt;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/hello", get(|| async { "Hello!" }))
        .spa();  // 👈 That's it!

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(heisenberg::shutdown_signal())
        .await
        .unwrap();
}
```

### 3. Run it

```bash
# Development mode (proxies to frontend dev server)
cargo run

# Production build (embeds assets in binary)
cargo heisenberg build --release
```

**That's it!** Heisenberg automatically:
- 🔍 Finds your frontend in `./web` or `./frontend`
- 🚀 Starts your dev server (`npm run dev`) in development
- 🔗 Proxies requests including WebSocket HMR
- 📦 Embeds assets in production builds
- 🌐 Opens your browser

## 📦 Cargo Plugin (Optional but Awesome)

Install the cargo plugin for the best experience:

```bash
cargo install cargo-heisenberg
```

### Commands

```bash
# Initialize config file (optional)
cargo heisenberg init

# Build frontend + backend in one command
cargo heisenberg build --release

# Dev mode with split-pane TUI (frontend + backend logs)
cargo heisenberg run
```

### Config File (Optional)

Create `heisenberg.toml` for custom setups:

```toml
[spa]
working_dir = "./web"
output_dir = "./web/build"
dev_command = "npm run dev"      # optional, auto-inferred
build_command = "npm run build"  # optional, auto-inferred
dev_server = "http://localhost:5173"  # optional, auto-inferred
```

## 🎯 API Examples

### Single SPA (Most Common)

```rust
use heisenberg::SpaExt;

let app = Router::new()
    .route("/api/hello", get(hello))
    .spa();  // Mounts at /*, auto-detects ./web or ./frontend
```

### Multiple SPAs

```rust
let app = Router::new()
    .route("/api/hello", get(hello))
    .spa_at("/admin/*")  // Smart: infers ./admin directory
    .spa_at("/app/*");   // Smart: infers ./app directory
```

### Explicit Control

```rust
let app = Router::new()
    .route("/api/hello", get(hello))
    .spa_at_from("/admin/*", "./frontend/admin")
    .spa_at_from("/app/*", "./frontend/user");
```

## 🔧 Mode Detection

| Build Command | Mode | Behavior |
|---------------|------|----------|
| `cargo run` | Proxy | Forward to dev server |
| `cargo build --release` | Embed | Serve embedded assets |
| `HEISENBERG_MODE=embed cargo run` | Embed | Force embed mode |
| `HEISENBERG_MODE=proxy cargo build --release` | Proxy | Force proxy mode |

## 🏗️ Framework Support

### Axum (Recommended)

```rust
use heisenberg::SpaExt;

let app = Router::new()
    .route("/api/hello", get(handler))
    .spa();
```

### Actix-web

```rust
use heisenberg::actix::serve_spa;

HttpServer::new(|| {
    App::new()
        .route("/api/hello", web::get().to(handler))
        .default_service(serve_spa("./dist"))
})
```

### Rocket

```rust
use heisenberg::rocket::serve_spa;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", routes![hello])
        .mount("/", serve_spa("./dist"))
}
```

## 📊 Examples

- **[axum-simple-v2](examples/axum-simple-v2/)** - ⭐ New API showcase
- **[axum-sveltekit](examples/axum-sveltekit/)** - SvelteKit with WebSocket HMR
- **[axum-multi-spa](examples/axum-multi-spa/)** - Multiple frontend apps
- **[actix-react](examples/actix-react/)** - Actix-web integration
- **[rocket-vue](examples/rocket-vue/)** - Rocket integration

## 🆚 Comparison

### Before Heisenberg

```rust
// 20+ lines of boilerplate
let assets = RustEmbed::new("./dist");
let proxy = if cfg!(debug_assertions) {
    Some(start_dev_server()?)
} else {
    None
};
// ... more setup code
```

### With Heisenberg

```rust
// 1 line
.spa()
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Try the showcase example
cd examples/axum-sveltekit && cargo run
```

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
