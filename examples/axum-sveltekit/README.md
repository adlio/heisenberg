# Axum-SvelteKit Example

Demonstrates Heisenberg with Axum and SvelteKit, featuring a TODO list with server-side state.

## Prerequisites

Install the `cargo-heisenberg` CLI tool:

```bash
# From crates.io (for your own projects)
cargo install cargo-heisenberg

# Or from local source (for running this example)
cargo install --path ../../cargo-heisenberg
```

## Zero-Config Setup

This example uses **no heisenberg.toml** file. Everything is inferred automatically:

- **Working directory**: `./web` (detected by finding `package.json`)
- **Output directory**: `./web/build` (SvelteKit default)
- **Dev command**: `npm run dev` (from package.json scripts)
- **Build command**: `npm run build` (from package.json scripts)
- **Dev server**: `http://localhost:5173` (from vite.config.js)

Heisenberg checks for frontend directories in this order: `./web`, `./frontend`, then root.

## Development (Proxy Mode)

```bash
cargo heisenberg run
```

**Server:** http://127.0.0.1:3001  
**API:** `/api/todos`  
**Frontend:** Vite dev server on port 5173 (proxied at `/*` with HMR)

## Production (Embed Mode)

```bash
cargo heisenberg build
cargo run
```

In embed mode, built frontend assets are served directly from the binary.

## Features

- **Server-side state**: Todos are stored in memory on the Axum backend
- **SvelteKit routing**: Client-side navigation between Home, About, and Todos pages
- **API integration**: Create and toggle todos via REST API
- **Educational content**: Pages explain how Heisenberg works in development and production modes