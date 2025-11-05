# Rocket-Vue Example

Demonstrates Heisenberg with Rocket and Vue 3.

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
**Frontend:** Vite dev server on port 3000 (proxied at `/*` with HMR)

## Production (Embed Mode)

```bash
cargo heisenberg build
cargo run
```

In embed mode, built frontend assets are served directly from the binary.
