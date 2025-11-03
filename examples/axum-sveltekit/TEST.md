# Testing the Seamless Experience

## What to Test

### 1. One-Command Start ✨
```bash
cd examples/axum-sveltekit
cargo run
```

**Expected:**
- Rust server starts on port 3001
- Heisenberg detects `web/package.json`
- Vite dev server starts automatically
- Browser opens to http://127.0.0.1:3001
- Todo app loads and works

### 2. WebSocket HMR 🔥
```bash
# While cargo run is running:
# Edit web/src/routes/+page.svelte
# Change a heading or add text
```

**Expected:**
- Changes appear instantly (< 1 second)
- No page refresh
- Todo state preserved
- Terminal shows Vite HMR update logs

### 3. API Integration 🔌
```bash
# In the browser:
# 1. Add a todo
# 2. Toggle completion
# 3. Check browser DevTools Network tab
```

**Expected:**
- POST /api/todos creates todo
- POST /api/todos/:id/toggle works
- Rust logs show API requests
- Frontend updates immediately

### 4. Production Build 📦
```bash
cargo build --release
./target/release/axum-sveltekit
```

**Expected:**
- Single binary runs
- No Vite server starts
- Assets served from embedded files
- App works identically
- No Node.js needed

## Success Criteria

✅ Zero manual steps (no `npm install`, no separate terminals)  
✅ HMR works through WebSocket proxy  
✅ Logs from Rust + Vite appear together  
✅ Browser opens automatically  
✅ Production build is single binary  

## Common Issues

**"npm: command not found"**
→ Install Node.js first

**Port 5173 already in use**
→ Kill existing Vite: `lsof -ti:5173 | xargs kill`

**HMR not working**
→ Check browser console for WebSocket errors
→ Verify Vite started (check terminal logs)
