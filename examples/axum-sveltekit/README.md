# Heisenberg + Axum + SvelteKit Example

**The showcase example** demonstrating Heisenberg's seamless developer experience.

## ✨ The Magic

```bash
cargo run
```

One command. Zero configuration. Full-stack development bliss.

Heisenberg automatically:
- Detects your frontend from `package.json`
- Installs dependencies if needed
- Starts Vite dev server
- Proxies WebSocket for HMR
- Merges logs from Rust + Vite

## Features

- **Zero Config**: No manual setup, works out of the box
- **WebSocket HMR**: Hot reload through transparent proxying
- **Unified Logs**: See both servers in one terminal
- **Production Ready**: Single binary with embedded assets
- **Modern Stack**: Svelte 5 + SvelteKit + Axum

## Quick Start

### Development Mode (Proxy)
```bash
# One command - that's it!
cargo run
```

Heisenberg automatically:
- ✅ Detects your `package.json` and finds `npm run dev`
- ✅ Starts the Vite dev server
- ✅ Proxies frontend requests (including WebSocket HMR)
- ✅ Opens your browser to http://127.0.0.1:3001

### Production Mode (Embed)
```bash
# Build frontend assets first
cd web && npm run build && cd ..

# Then build Rust binary (assets are embedded during compilation)
cargo build --release
./target/release/axum-sveltekit
```

The Rust binary contains embedded assets - completely standalone, no external files or Node.js required!

## Architecture

### Backend (Rust + Axum)
- Simple in-memory todo store with Arc<Mutex<HashMap>>
- RESTful API endpoints:
  - `GET /api/todos` - List all todos
  - `POST /api/todos` - Create new todo
  - `POST /api/todos/:id/toggle` - Toggle todo completion
- Heisenberg layer handles SPA serving automatically

### Frontend (SvelteKit)
- Modern Svelte 5 with runes ($state, $effect)
- Client-side routing between home and about pages
- Responsive design with clean CSS
- Fetch-based API integration
- Static adapter for SPA deployment

## How It Works

### Development Mode
1. `cargo run` starts the Rust server
2. Heisenberg detects `web/package.json` and runs `npm run dev`
3. Vite dev server starts on port 5173
4. Frontend requests → proxied to Vite
5. API requests → handled by Rust backend
6. WebSocket HMR → proxied transparently for hot reload
7. Browser opens automatically

### Production Mode
1. Frontend assets must be built first: `cd web && npm run build`
2. Assets are embedded into the Rust binary during compilation
3. SPA fallback ensures client-side routing works
4. Completely standalone - no external files or Node.js required

## Project Structure

```
axum-sveltekit/
├── Cargo.toml              # Rust dependencies
├── src/main.rs             # Axum server with Heisenberg
└── web/                    # SvelteKit frontend
    ├── package.json        # Frontend dependencies
    ├── svelte.config.js    # SvelteKit configuration
    └── src/routes/         # SvelteKit pages
        ├── +layout.js      # SPA configuration
        ├── +page.svelte    # Todo app (home page)
        └── about/
            └── +page.svelte # About page
```

## Key Configuration

### Asset Embedding
```rust
// At the top of main.rs - embeds assets at compile time
heisenberg::embed_spa_assets!("./web/build");
```

**The path must match your build output directory and the `.spa()` configuration below.**

### Heisenberg Setup
```rust
let config = heisenberg::Heisenberg::new()
    .spa("./web/build")  // Must match embed_spa_assets! path
    .build();

let app = Router::new()
    .nest("/api", api_routes)
    .layer(heisenberg::HeisenbergLayer::new(config));
```

### SvelteKit Static Adapter
```javascript
import adapter from '@sveltejs/adapter-static';

export default {
    kit: {
        adapter: adapter({
            pages: 'build',      // Output directory
            assets: 'build',
            fallback: 'index.html'  // SPA fallback
        })
    }
};
```

## Customizing for Your Setup

### Custom Build Command
If you use a different build command (e.g., `npm run prod`):

```bash
# Run your custom build command first
cd web && npm run prod && cd ..

# Then build Rust binary (assets embedded during compilation)
cargo build --release
```

The `embed_spa_assets!()` macro path must point to wherever your build outputs files.

### Custom Output Directory
If your build outputs to a different location:

```rust
// In src/main.rs - both paths must match
heisenberg::embed_spa_assets!("./web/dist");

let config = heisenberg::Heisenberg::new()
    .spa("./web/dist")
    .build();
```

```javascript
// In svelte.config.js
adapter: adapter({
    pages: 'dist',  // Must match paths above
    assets: 'dist',
    fallback: 'index.html'
})
```

### Required Dependencies
Your Cargo.toml needs these for asset embedding:

```toml
[dependencies]
heisenberg = { version = "0.2", features = ["tower"] }
rust-embed = "8.0"
ctor = "0.2"
```

## WebSocket HMR Proxying

Heisenberg automatically proxies WebSocket connections for Vite's Hot Module Replacement:

- Vite's HMR WebSocket connects through the Rust proxy
- Changes to `.svelte` files trigger instant updates
- No page refresh needed
- Works transparently - zero configuration

## Verifying It Works

### Development Mode
```bash
cargo run
```

**Expected behavior:**
- Rust server starts on port 3001
- Heisenberg detects `web/package.json` and runs `npm run dev`
- Vite dev server starts automatically on port 5173
- Browser opens to http://127.0.0.1:3001
- Todo app loads and is fully functional

### Hot Module Replacement
While `cargo run` is running, edit `web/src/routes/+page.svelte` (change a heading or add text).

**Expected behavior:**
- Changes appear instantly (< 1 second)
- No page refresh required
- Todo state is preserved
- Terminal shows Vite HMR update logs

### API Integration
In the browser:
1. Add a todo item
2. Toggle its completion status
3. Check browser DevTools Network tab

**Expected behavior:**
- `POST /api/todos` creates the todo
- `POST /api/todos/:id/toggle` toggles completion
- Rust logs show API requests
- Frontend updates immediately

### Production Build
```bash
# IMPORTANT: Build frontend first
cd web && npm run build && cd ..

# Then build Rust binary (assets embedded during compilation)
cargo build --release
./target/release/axum-sveltekit
```

**Expected behavior:**
- Assets are embedded in the binary during compilation
- No Vite server starts
- Assets served from memory (embedded in binary)
- App works identically to development mode
- Completely standalone - no external files or Node.js required

## Troubleshooting

**"npm: command not found"**
→ Install Node.js first

**Port 5173 already in use**
→ Kill existing Vite: `lsof -ti:5173 | xargs kill`

**HMR not working**
→ Check browser console for WebSocket errors
→ Verify Vite started (check terminal logs)