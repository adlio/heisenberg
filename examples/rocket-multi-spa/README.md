# Rocket Multi-SPA Example

Demonstrates serving multiple SPAs with Rocket.

## Prerequisites

Install the `cargo-heisenberg` CLI tool:

```bash
# From crates.io (for your own projects)
cargo install cargo-heisenberg

# Or from local source (for running this example)
cargo install --path ../../cargo-heisenberg
```

## Development (Proxy Mode)

```bash
cargo heisenberg run
```

**Server:** http://127.0.0.1:8000
**API:** `/api/hello`
**Frontends:**
- Main app: Vite dev server on port 3000 (proxied at `/`)
- Admin app: Vite dev server on port 3001 (proxied at `/admin/*`)

> **Note:** The Rocket adapter does not currently support WebSocket proxying, so Hot Module Reload (HMR) won't work. You'll need to manually refresh the page to see changes. For full HMR support, use the Axum integration.

## Production (Embed Mode)

```bash
cargo heisenberg build
cargo run
```

In embed mode, built assets from both SPAs are served directly from the binary.
