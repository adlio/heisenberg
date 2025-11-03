# WebSocket Demo - Quick Start

## 30-Second Test

```bash
# Terminal 1
cargo run --bin websocket-backend

# Terminal 2  
cargo run --bin websocket-demo

# Browser
open http://localhost:3000
```

Type messages, see them echo back!

## What's Happening

```
Browser → Heisenberg (port 3000) → Backend (port 8080) → Echo back
```

## Verify It's Working

✅ Backend shows: `📥 New connection from 127.0.0.1:xxxxx`  
✅ Browser shows: `Connected ✓`  
✅ Messages appear in both sent/received sections  
✅ Backend logs each message received  

## Troubleshooting

**"Disconnected ✗"**  
→ Start backend first: `cargo run --bin websocket-backend`

**Port already in use**  
→ Kill existing process: `lsof -ti:8080 | xargs kill`

**No echo response**  
→ Check backend terminal for errors
