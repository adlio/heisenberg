# Rocket-Vue Example

Demonstrates Heisenberg's Rocket adapter with a Vue-style frontend.

## Running

### Production Mode (Static Files)
```bash
cd examples/rocket-vue
cargo run
```
Then visit http://127.0.0.1:8000

### Development Mode (Live Vue + HMR)
```bash
# Terminal 1: Start Rocket backend
cd examples/rocket-vue
NODE_ENV=development cargo run

# Terminal 2: Start Vue dev server
npm install
npm run dev
```
Then visit http://localhost:3000 (Vue dev server with API proxy)

## What it demonstrates

- Heisenberg Rocket adapter integration
- API routes served by Rust backend (`/api/hello`)
- Static frontend served by Heisenberg adapter
- Dual-mode operation (development/production)

## API Endpoints

- `GET /api/hello` - Returns JSON response from Rocket backend

## Testing HMR (Hot Module Reload)

In development mode, this example would proxy to a Vue dev server.
For manual testing:

1. Start a Vue dev server on port 3000 (if available)
2. Configure the example to proxy to `http://localhost:3000`
3. Verify that changes to Vue components are reflected immediately

## Current Mode

This example demonstrates the adapter functionality with static files.
The dual-mode behavior (dev proxy vs prod embedded) is handled by the adapter.

## Rocket Specifics

- Uses Rocket's default port 8000
- Leverages Rocket's JSON macros and routing
- Demonstrates different adapter pattern from Axum/Tower approach