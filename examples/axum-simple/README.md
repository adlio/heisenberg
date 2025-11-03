# Axum Simple Example

A minimal example demonstrating Heisenberg with Axum.

## Running

```bash
cd examples/axum-simple
cargo run
```

Then visit http://localhost:3000

## What it demonstrates

- Basic Heisenberg integration with Axum using Tower layer
- API routes served by Rust backend (`/api/hello`)
- Static frontend served by Heisenberg (currently from `./dist/`)
- Frontend can call backend API endpoints

## Mode Detection

This example automatically switches between:
- **Proxy mode** (`cargo run`) - forwards to frontend dev server
- **Embed mode** (`cargo build --release`) - serves embedded static assets

The static files are served from the `dist/` directory in both modes.
