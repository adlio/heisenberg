# Axum Multi-SPA Example

Demonstrates serving multiple SPAs from a single Axum backend, featuring:

- **Minimal config** - `heisenberg.toml` only needs `name` and `working_dir`; commands and output directories are inferred
- **Flexible mount points** - SPAs can be mounted at paths different from their directory names (e.g., `user-webapp/` mounted at `/`, `admin-webapp/` mounted at `/admin`)

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

**Server:** http://127.0.0.1:3002  
**API:** `/api/hello`  
**Frontends:**
- `user-webapp/`: Vite dev server on port 5173 (proxied at `/`)
- `admin-webapp/`: Vite dev server on port 5174 (proxied at `/admin/*`)

## Production (Embed Mode)

```bash
cargo heisenberg build
cargo run
```

In embed mode, built assets from both SPAs are served directly from the binary.
