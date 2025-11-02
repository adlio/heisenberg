# WebSocket Proxying Demo

This example demonstrates Heisenberg's WebSocket proxying capabilities in development mode.

## Features

- WebSocket echo server backend
- Real-time bidirectional communication
- Automatic proxying from Heisenberg to backend WebSocket server
- Simple HTML/JS frontend for testing

## Running the Example

1. Start the backend WebSocket server:
   ```bash
   cargo run --example websocket-backend
   ```

2. In another terminal, start the Heisenberg proxy server:
   ```bash
   cargo run --example websocket-demo
   ```

3. Open http://localhost:3000 in your browser

4. Type messages in the input box and see them echoed back

## How It Works

- Backend runs a WebSocket echo server on port 8080
- Heisenberg proxies WebSocket upgrade requests to the backend
- Frontend connects to Heisenberg, which transparently proxies to backend
- Messages flow: Browser ↔ Heisenberg ↔ Backend
