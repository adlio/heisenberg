# Heisenberg + Axum + SvelteKit Example

A minimal but realistic todo application demonstrating Heisenberg's dual-mode serving capabilities with modern SvelteKit and Axum.

## Features Demonstrated

- **Dual-mode serving**: Seamless switching between development (proxy) and production (embedded) modes
- **SvelteKit integration**: Client-side routing with SPA fallback support
- **Modern SvelteKit**: Uses Svelte 5 runes and latest SvelteKit features
- **API integration**: RESTful JSON API with Axum backend
- **Hot Module Replacement**: Full HMR support in development mode
- **Single binary deployment**: Production builds embed all assets

## Quick Start

```bash
# Development mode (proxy + HMR)
cargo run

# Production mode (test embedded assets)
HEISENBERG_MODE=embed cargo run

# Build for production
cargo build --release
```

Visit http://127.0.0.1:3000 to see the todo app in action.

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
2. Heisenberg automatically starts the Vite dev server (`npm run dev`)
3. Frontend requests are proxied to Vite (port 5173)
4. API requests go directly to Rust backend
5. Full HMR and fast refresh enabled

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
    .layer(heisenberg::Heisenberg::new().spa("./web/build"));
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

This example showcases Heisenberg's core value proposition: write once, deploy anywhere with automatic mode detection and zero configuration complexity.