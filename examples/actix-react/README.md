# Actix-React Example

Demonstrates Heisenberg's Actix-web adapter with a React-style frontend.

## Running

```bash
cd examples/actix-react
cargo run
```

Then visit http://127.0.0.1:8080

## What it demonstrates

- Heisenberg Actix-web adapter integration
- API routes served by Rust backend (`/api/hello`)
- Static frontend served by Heisenberg adapter
- Dual-mode operation (development/production)

## API Endpoints

- `GET /api/hello` - Returns JSON response from Actix-web backend

## Testing HMR (Hot Module Reload)

In development mode, this example would proxy to a React dev server.
For manual testing:

1. Start a React dev server on port 3000 (if available)
2. Configure the example to proxy to `http://localhost:3000`
3. Verify that changes to React components are reflected immediately

## Current Mode

This example demonstrates the adapter functionality with static files.
The dual-mode behavior (dev proxy vs prod embedded) is handled by the adapter.