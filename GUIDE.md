# Heisenberg User Guide

Heisenberg is a framework-agnostic dual-mode web serving library for Rust applications. It seamlessly switches between development mode (proxying to frontend dev servers) and production mode (serving embedded static assets).

## Table of Contents

- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Mode Detection](#mode-detection)
- [Framework Integration](#framework-integration)
- [Troubleshooting](#troubleshooting)
- [Performance Tuning](#performance-tuning)

## Quick Start

### 1. Add Heisenberg to your project

```toml
[dependencies]
heisenberg = "0.1"
axum = "0.7"
tokio = { version = "1.35", features = ["full"] }
```

### 2. Basic setup with Axum

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
# Development mode (proxy to frontend dev server)
cargo run

# Production mode (serve embedded assets)
cargo build --release && ./target/release/your-app
```

## Configuration

### Smart Inference

Heisenberg automatically infers configuration from your project structure:

```rust
// Infers everything from ./web/dist
let config = Heisenberg::new()
    .spa("./web/dist")  // Looks for ./web/package.json
    .build();           // Extracts dev command and port
```

**Inference Logic:**
1. Strip build directory name (`dist`, `build`, `out`, `public`, `www`) from path
2. Search for `package.json` in resulting directory (walks up if needed)
3. Extract dev command from scripts: `dev` > `start` > `serve`
4. Detect common dev server ports: 5173, 3000, 8080

### Manual Configuration

Override any inferred settings:

```rust
let config = Heisenberg::new()
    .spa("./frontend/dist")
        .dev_server("http://localhost:3000")
        .dev_command(["npm", "run", "dev"])
        .working_dir("./frontend")
        .fallback_file("index.html")
        .open_browser(true)
    .build();
```

### Multiple SPAs

Support micro-frontend architectures:

```rust
let config = Heisenberg::new()
    .spa("/admin/*", "./admin/dist")
        .dev_server("http://localhost:3001")
    .spa("/*", "./app/dist")
        .dev_server("http://localhost:3000")
        .fallback_file("index.html")
    .build();
```

## Mode Detection

### Automatic Detection

Heisenberg automatically detects the appropriate mode:

- **Development**: `cargo run` → Proxy mode (fast iteration)
- **Production**: `cargo build --release` → Embed mode (single binary)

### Manual Override

Use environment variables to override:

```bash
# Force embed mode in debug builds (test production behavior)
HEISENBERG_MODE=embed cargo run

# Force proxy mode in release builds (fast release testing)
HEISENBERG_MODE=proxy cargo build --release
```

### Build Script Integration

Add to your `build.rs` for automatic frontend builds:

```rust
fn main() {
    println!("cargo:rerun-if-env-changed=HEISENBERG_MODE");
    
    let mode = std::env::var("HEISENBERG_MODE")
        .unwrap_or_else(|_| {
            if std::env::var("PROFILE").unwrap() == "release" {
                "embed".to_string()
            } else {
                "proxy".to_string()
            }
        });
    
    if mode == "embed" {
        // Run frontend build
        std::process::Command::new("npm")
            .args(&["run", "build"])
            .current_dir("./frontend")
            .status()
            .expect("Failed to build frontend");
    }
}
```

## Framework Integration

### Tower-based Frameworks (Axum, Warp, Hyper)

Works automatically via Tower layer:

```rust
// Axum
let app = Router::new()
    .route("/api/*", get(api_handler))
    .layer(heisenberg_config);

// Warp
let routes = warp::path("api")
    .and(api_routes)
    .or(warp::any().map(|| warp::reply()))
    .with(heisenberg_config);
```

### Actix-web

Use the helper function approach:

```toml
[dependencies]
heisenberg = { version = "0.1", features = ["actix"] }
```

```rust
use heisenberg::actix::serve_spa;

async fn spa_handler(req: HttpRequest) -> impl Responder {
    serve_spa(&req, &heisenberg_config).await
}

App::new()
    .route("/api/*", web::get().to(api_handler))
    .route("/*", web::get().to(spa_handler))
```

### Rocket

Use the helper function approach:

```toml
[dependencies]
heisenberg = { version = "0.1", features = ["rocket"] }
```

```rust
use heisenberg::rocket::serve_spa;

#[get("/<path..>")]
async fn spa_handler(path: PathBuf) -> Result<NamedFile, Status> {
    serve_spa(&path, &heisenberg_config).await
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", routes![api_handler])
        .mount("/", routes![spa_handler])
}
```

## Troubleshooting

### Enable Logging

Add structured logging for debugging:

```toml
[dependencies]
heisenberg = { version = "0.1", features = ["logging"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Your app code...
}
```

```bash
# Run with debug logging
RUST_LOG=debug,heisenberg=trace cargo run
```

### Common Issues

#### Frontend dev server not starting

**Symptoms**: "Frontend dev server unavailable" error page

**Solutions**:
1. Check dev command: `.dev_command(["npm", "run", "dev"])`
2. Verify working directory: `.working_dir("./frontend")`
3. Ensure package.json exists with dev script
4. Check port conflicts: `.dev_server("http://localhost:3000")`

#### Assets not found in production

**Symptoms**: 404 errors for static assets

**Solutions**:
1. Verify embed directory exists: `./dist` should contain built assets
2. Check build script runs: Add `build.rs` to run frontend build
3. Ensure assets are built: `npm run build` before `cargo build --release`
4. Verify fallback file: `.fallback_file("index.html")`

#### Route conflicts

**Symptoms**: API routes not working

**Solutions**:
1. Order routes correctly: API routes before SPA routes
2. Use specific patterns: `/api/*` before `/*`
3. Check route validation errors in logs

### Debug Mode Detection

```rust
use heisenberg::core::mode::detect_mode;

println!("Current mode: {:?}", detect_mode());
```

## Performance Tuning

### Development Mode

- **Fast builds**: Debug builds default to proxy mode
- **Hot reload**: Frontend changes don't require Rust recompilation
- **Process management**: Automatic dev server startup and health checking

### Production Mode

- **Single binary**: All assets embedded, no external dependencies
- **Optimized serving**: Efficient static file serving with proper MIME types
- **Caching headers**: Long-term caching for embedded assets

### Memory Usage

- **Lazy loading**: Assets loaded on-demand
- **Connection pooling**: Reused HTTP connections for proxy requests
- **Efficient routing**: Cached path matching for repeated requests

### Build Time Optimization

```bash
# Fast development builds (proxy mode)
cargo run

# Optimized release builds (embed mode)
cargo build --release

# Test production behavior without full optimization
HEISENBERG_MODE=embed cargo run
```

### Monitoring

Enable logging to monitor performance:

```bash
RUST_LOG=info,heisenberg=debug cargo run
```

Key metrics logged:
- Route matching performance
- Dev server health check timing
- Process startup duration
- Request routing decisions

## Migration Guide

### From Manual Proxy Setup

**Before:**
```rust
// Manual proxy logic
if cfg!(debug_assertions) {
    // Proxy to localhost:3000
} else {
    // Serve from ./dist
}
```

**After:**
```rust
let app = Router::new()
    .layer(Heisenberg::new().spa("./dist"));
```

### From Static File Serving

**Before:**
```rust
let app = Router::new()
    .nest_service("/", ServeDir::new("./dist"));
```

**After:**
```rust
let app = Router::new()
    .layer(Heisenberg::new().spa("./dist"));
```

Benefits: Automatic dev mode, process management, SPA routing support.