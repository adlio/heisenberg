# ✨ Seamless Development Experience Demo

## One Command to Rule Them All

```bash
cargo run
```

That's it! Heisenberg automatically:

1. ✅ Detects `web/package.json` 
2. ✅ Runs `npm install` if needed (first time only)
3. ✅ Starts Vite dev server on port 5173
4. ✅ Proxies frontend requests (including WebSocket HMR)
5. ✅ Serves your Rust API on `/api/*`
6. ✅ Merges logs from both servers

## What You Get

- **Hot Module Replacement**: Edit `.svelte` files, see changes instantly
- **WebSocket Proxying**: HMR works transparently through Heisenberg
- **Unified Logs**: See Rust + Vite output together
- **Zero Config**: No manual setup, no separate terminals

## Test HMR

While `cargo run` is running:

1. Edit `web/src/routes/+page.svelte`
2. Change a heading or add text
3. Watch it update instantly in browser (no refresh!)

## Production Build

```bash
cargo build --release
./target/release/axum-sveltekit
```

Single binary with embedded assets - ship it!

## Architecture

```
Browser Request
    ↓
Heisenberg (port 3001)
    ├─→ /api/* → Rust/Axum handlers
    └─→ /* → Proxy to Vite (port 5173)
            ├─→ HTTP requests
            └─→ WebSocket (HMR)
```

## Success Indicators

✅ Rust server starts on port 3001  
✅ Vite starts automatically on port 5173  
✅ Browser loads app from http://127.0.0.1:3001  
✅ Todo app works (add/toggle todos)  
✅ HMR updates without page refresh  
✅ Both server logs appear in terminal  

## Troubleshooting

**Vite not starting?**
```bash
# Check if it's running
lsof -ti:5173

# Check for errors
cd web && npm run dev
```

**Port conflicts?**
```bash
# Kill existing processes
lsof -ti:3001 | xargs kill
lsof -ti:5173 | xargs kill
```

**Dependencies missing?**
```bash
cd web && npm install
```
