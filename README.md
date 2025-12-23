# Heisenberg

[![Crates.io](https://img.shields.io/crates/v/heisenberg.svg)](https://crates.io/crates/heisenberg)
[![Documentation](https://docs.rs/heisenberg/badge.svg)](https://docs.rs/heisenberg)
[![codecov](https://codecov.io/gh/adlio/heisenberg/branch/main/graph/badge.svg)](https://codecov.io/gh/adlio/heisenberg)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Heisenberg serves SPAs from Rust web applications. It switches between proxy mode (forwarding to a frontend dev server) and embed mode (serving assets compiled into your binary).

## How It Works

In development, run `cargo heisenberg run`. This starts your frontend dev server and your Rust backend, proxying frontend requests (including WebSocket HMR) to the dev server.

For release builds, run `cargo heisenberg build --release`. This builds your frontend, then compiles the assets into your Rust binary using the `embed_spa!` macro.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
heisenberg = "0.4"
axum = "0.7"
tokio = { version = "1.35", features = ["full"] }
rust-embed = "8.0"
```

Write your server:

```rust
use axum::{routing::get, Router};
use heisenberg::{Heisenberg, HeisenbergLayer};

#[tokio::main]
async fn main() {
    let spa = heisenberg::embed_spa!();
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server("http://localhost:5173")
        .build();

    let app = Router::new()
        .route("/api/hello", get(|| async { "Hello!" }))
        .layer(HeisenbergLayer::new(config));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(heisenberg::shutdown_signal())
        .await
        .unwrap();
}
```

Install and run the cargo plugin:

```bash
cargo install cargo-heisenberg

# Development (proxy mode with HMR)
cargo heisenberg run

# Release build (embeds assets)
cargo heisenberg build --release
```

## Cargo Plugin Commands

```bash
cargo heisenberg init    # Generate heisenberg.toml
cargo heisenberg build   # Build frontend, then cargo build
cargo heisenberg run     # Start frontend + backend with split-pane TUI
```

Add `--no-tui` to `cargo heisenberg run` for plain output (useful for copying error messages).

## Configuration

### When You Don't Need heisenberg.toml

The plugin auto-detects your frontend if you have a single SPA in `./web` or `./frontend`. No config file needed.

### When You Need heisenberg.toml

Create `heisenberg.toml` when you have:
- Multiple SPAs
- A frontend in a non-standard directory
- Custom build or dev commands

Single SPA example:

```toml
[spa]
working_dir = "./client"
output_dir = "./client/dist"
```

Multiple SPAs:

```toml
[[spa]]
name = "app"
working_dir = "./app"
output_dir = "./app/dist"
dev_server = "http://localhost:5173"

[[spa]]
name = "admin"
working_dir = "./admin"
output_dir = "./admin/dist"
dev_server = "http://localhost:5174"
```

In your Rust code, reference named SPAs:

```rust
let app = heisenberg::embed_spa!("app");
let admin = heisenberg::embed_spa!("admin");

let config = Heisenberg::new()
    .route("/admin/*", admin)
    .route("/*", app)
    .build();
```

## Mode Detection

| Command | Mode | Behavior |
|---------|------|----------|
| `cargo heisenberg run` | Proxy | Forwards to dev server with HMR |
| `cargo run` | Embed | Serves embedded assets |
| `cargo build --release` | Embed | Compiles assets into binary |
| `HEISENBERG_MODE=proxy cargo run` | Proxy | Force proxy mode |
| `HEISENBERG_MODE=embed cargo run` | Embed | Force embed mode |

## Framework Support

### Axum

```rust
let spa = heisenberg::embed_spa!();
let config = Heisenberg::new()
    .route("/*", spa)
    .build();

let app = Router::new()
    .route("/api/hello", get(handler))
    .layer(HeisenbergLayer::new(config));
```

### Actix-web

```rust
let spa = heisenberg::embed_spa!();
let config = Heisenberg::new()
    .route("/*", spa)
    .build();

HttpServer::new(move || {
    App::new()
        .app_data(web::Data::new(config.clone()))
        .route("/api/hello", web::get().to(handler))
        .default_service(web::to(heisenberg::adapters::actix::serve_spa))
})
```

### Rocket

```rust
let spa = heisenberg::embed_spa!();
let config = Heisenberg::new()
    .route("/*", spa)
    .build();

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(config)
        .mount("/api", routes![hello])
        .mount("/", spa_routes())
}
```

## Examples

- [axum-sveltekit](examples/axum-sveltekit/) - Axum with SvelteKit
- [axum-multi-spa](examples/axum-multi-spa/) - Multiple SPAs with Axum
- [actix-react](examples/actix-react/) - Actix-web with React
- [rocket-vue](examples/rocket-vue/) - Rocket with Vue
- [rocket-multi-spa](examples/rocket-multi-spa/) - Multiple SPAs with Rocket

## License

MIT License. See [LICENSE](LICENSE).
