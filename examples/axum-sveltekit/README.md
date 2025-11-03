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

```bash
# One command - that's it!
cargo run
```

Heisenberg automatically:
- ✅ Detects your `package.json` and finds `npm run dev`
- ✅ Starts the Vite dev server
- ✅ Proxies frontend requests (including WebSocket HMR)
- ✅ Opens your browser to http://127.0.0.1:3001

**Production mode:**
```bash
cargo build --release
./target/release/axum-sveltekit
```

Single binary with embedded assets - no Node.js required!

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
1. Frontend assets are built and embedded during compilation
2. Single binary contains both backend and frontend
3. SPA fallback ensures client-side routing works
4. No external dependencies required for deployment

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

### Heisenberg Setup
```rust
let app = Router::new()
    .nest("/api", api_routes)
    .layer(heisenberg::HeisenbergLayer::new(
        heisenberg::Heisenberg::new().spa("./web/build").build()
    ));
```

### SvelteKit Static Adapter
```javascript
import adapter from '@sveltejs/adapter-static';

export default {
    kit: {
        adapter: adapter({
            pages: 'build',
            assets: 'build',
            fallback: 'index.html'  // SPA fallback
        })
    }
};
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
cargo build --release
./target/release/axum-sveltekit
```

**Expected behavior:**
- Single binary runs without external dependencies
- No Vite server starts
- Assets served from embedded files
- App works identically to development mode
- No Node.js required

## Troubleshooting

**"npm: command not found"**
→ Install Node.js first

**Port 5173 already in use**
→ Kill existing Vite: `lsof -ti:5173 | xargs kill`

**HMR not working**
→ Check browser console for WebSocket errors
→ Verify Vite started (check terminal logs)