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

## Current Mode

This example currently runs in **development mode** (proxy mode is not yet implemented).
The static files are served directly from the `dist/` directory.

## Next Steps

In Phase 3, this will be enhanced with:
- Automatic mode detection (dev vs prod)
- Frontend dev server proxy support
- Process management for frontend builds
