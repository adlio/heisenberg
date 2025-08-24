# Axum Multi-SPA Example

This example demonstrates Heisenberg's ability to serve multiple Single Page Applications (SPAs) from different route patterns, each with their own configuration and API endpoints.

## Architecture

This example showcases a micro-frontend architecture with:

- **Admin Panel** (`/admin/*`) - System administration interface
- **Main Application** (`/app/*`) - Primary user application  
- **Landing Page** (`/*`) - Default catch-all route

Each SPA can be developed independently with different frontend frameworks and build processes, while sharing a common Rust backend.

## Features Demonstrated

### Multiple SPA Routes
- Different route patterns with priority handling
- Route-specific configuration (dev servers, build directories, etc.)
- Fallback file support for client-side routing

### Advanced Configuration
- Custom dev server URLs and commands for each SPA
- Different working directories for frontend projects
- Selective browser opening (only for main landing page)

### Shared Backend APIs
- Service-specific API endpoints (`/api/admin`, `/api/app`)
- Shared global endpoints (`/api/status`)
- RESTful user management (`/api/admin/users/:id`)

## Running the Example

### Production Mode (Embedded Assets)
```bash
# Build and run with embedded static files
cargo run --release
```

### Development Mode (Proxy to Dev Servers)
```bash
# Run in development mode (will proxy to frontend dev servers)
cargo run
```

In development mode, Heisenberg will attempt to proxy requests to:
- Admin Panel: `http://localhost:3001` (from `./admin-frontend`)
- Main App: `http://localhost:3002` (from `./app-frontend`) 
- Landing Page: `http://localhost:3000` (from `./landing-frontend`)

## Testing the Example

1. **Start the server:**
   ```bash
   cargo run
   ```

2. **Visit the different SPAs:**
   - Landing Page: http://localhost:8081/
   - Admin Panel: http://localhost:8081/admin/
   - Main App: http://localhost:8081/app/

3. **Test API endpoints:**
   - System Status: http://localhost:8081/api/status
   - Admin API: http://localhost:8081/api/admin
   - App API: http://localhost:8081/api/app
   - User API: http://localhost:8081/api/admin/users/1

4. **Test client-side routing:**
   - Navigate to http://localhost:8081/admin/users (should serve admin SPA)
   - Navigate to http://localhost:8081/app/profile (should serve main app SPA)
   - Navigate to http://localhost:8081/anything (should serve landing page)

## Route Priority

Heisenberg handles overlapping routes with a priority system:

1. **Exact matches** (highest priority): `/admin/users`
2. **Prefix matches** (medium priority): `/admin/*`, `/app/*`  
3. **Catch-all matches** (lowest priority): `/*`

In this example:
- `/admin/dashboard` → Admin Panel SPA
- `/app/profile` → Main App SPA
- `/anything-else` → Landing Page SPA

## Configuration Breakdown

```rust
let heisenberg_config = Heisenberg::new()
    // Admin panel configuration
    .spa("./admin-dist")
    .pattern("/admin/*")
    .dev_server("http://localhost:3001")
    .dev_command(["npm", "run", "dev:admin"])
    .working_dir("./admin-frontend")
    .fallback_file("index.html")
    .open_browser(false)
    
    // Main app configuration  
    .spa("./app-dist")
    .pattern("/app/*")
    .dev_server("http://localhost:3002")
    .dev_command(["npm", "run", "dev:app"])
    .working_dir("./app-frontend")
    .fallback_file("index.html")
    .open_browser(false)
    
    // Landing page (catch-all)
    .spa("./landing-dist")
    .pattern("/*")
    .dev_server("http://localhost:3000")
    .dev_command(["npm", "run", "dev"])
    .working_dir("./landing-frontend")
    .fallback_file("index.html")
    .open_browser(true)
    .build(); // Only open browser for main entry point
```

## Real-World Usage

This pattern is ideal for:

- **Micro-frontends**: Different teams can develop separate SPAs
- **Role-based interfaces**: Admin vs user interfaces
- **Feature separation**: Different apps for different product areas
- **Technology diversity**: Each SPA can use different frontend frameworks
- **Independent deployment**: SPAs can be built and deployed separately

## Next Steps

To extend this example:

1. **Add real frontend projects** in the working directories
2. **Implement authentication** with role-based routing
3. **Add WebSocket support** for real-time features
4. **Configure CI/CD** for independent SPA deployments
5. **Add monitoring** and health checks for each service