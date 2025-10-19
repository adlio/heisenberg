# Heisenberg TODO

Critical gaps preventing the library from being functional.

## 🚨 Critical - Blocking Basic Functionality

### 1. Tower Service Implementation (`src/tower/service.rs`)
**Status:** Placeholder only  
**Lines:** 69-82

The core service implementation has a TODO comment and doesn't actually serve files or proxy requests:

```rust
if let Some(_route_config) = route_match {
    // TODO: Handle SPA routing (proxy/static files) in next phase
    // For now, return a placeholder response
    // ...
    // This is a type conversion issue - we need to handle it properly
    // For now, fall through to inner service
    inner_service.call(req).await
}
```

**What's needed:**
- [ ] Implement actual routing logic based on `Mode` (Development vs Production)
- [ ] Call `ProxyService::proxy_request()` in Development mode
- [ ] Call `StaticFileService::serve_file()` in Production mode
- [ ] Handle response type conversions properly (hyper Body types)
- [ ] Implement SPA fallback logic (serve index.html for non-existent routes)

### 2. Static File Service (`src/services/static_files.rs`)
**Status:** Stub implementation  
**Lines:** 20-36

Currently returns hardcoded HTML instead of serving actual files:

```rust
pub fn serve_file(&self, path: &str) -> Result<Response<String>, HeisenbergError> {
    // For now, just return a simple response
    // Will be enhanced with actual rust-embed integration
    if path == "/" || path == "/index.html" {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html")
            .body("<html><body><h1>Heisenberg Static Server</h1></body></html>".to_string())
            .unwrap())
    } else {
        Err(HeisenbergError::file_not_found(path, "..."))
    }
}
```

**What's needed:**
- [ ] Integrate `rust-embed` to actually embed static files at compile time
- [ ] Read files from the embedded assets
- [ ] Use `detect_mime_type()` method to set correct Content-Type headers
- [ ] Implement SPA fallback: serve `index.html` for 404s when `fallback_file` is set
- [ ] Handle binary files (images, fonts, etc.) not just text
- [ ] Add proper error handling for missing files

### 3. Response Type Compatibility
**Status:** Type mismatch issues

The services return `Response<String>` but Tower/Axum expect `Response<Body>`:

**What's needed:**
- [ ] Update `StaticFileService::serve_file()` to return `Response<Body>`
- [ ] Update `ProxyService::proxy_request()` to return `Response<Body>`
- [ ] Handle streaming responses for large files
- [ ] Properly convert between hyper body types

## 🔧 Important - Needed for Production Use

### 4. Process Management (`src/services/process.rs`)
**Status:** Unknown - needs verification

**What's needed:**
- [ ] Verify dev server process spawning works
- [ ] Test process lifecycle (start, monitor, stop)
- [ ] Handle process crashes and restarts
- [ ] Capture and log dev server output

### 5. Health Checking (`src/services/health.rs`)
**Status:** Unknown - needs verification

**What's needed:**
- [ ] Verify health check implementation works
- [ ] Test with various dev servers (Vite, webpack-dev-server, etc.)
- [ ] Add configurable health check endpoints
- [ ] Handle slow-starting dev servers

### 6. Proxy Service Enhancements (`src/services/proxy.rs`)
**Status:** Basic implementation exists

**What's needed:**
- [ ] Forward request headers properly
- [ ] Handle WebSocket connections for HMR
- [ ] Stream large responses instead of buffering
- [ ] Preserve response headers from dev server
- [ ] Handle redirects correctly

## 📚 Nice to Have - Enhanced Features

### 7. Configuration Inference
**Status:** Partially implemented

**What's needed:**
- [ ] Test package.json parsing
- [ ] Verify dev command detection works
- [ ] Add support for more frontend frameworks
- [ ] Handle monorepo structures

### 8. Browser Auto-Open
**Status:** Unknown

**What's needed:**
- [ ] Verify browser opening works on all platforms
- [ ] Make it configurable
- [ ] Handle cases where browser can't be opened

### 9. Logging and Diagnostics
**Status:** Partial - behind feature flag

**What's needed:**
- [ ] Add more debug logging throughout
- [ ] Log file serving operations
- [ ] Log proxy operations
- [ ] Add performance metrics

## 🧪 Testing

### 10. Integration Tests
**Status:** Unknown

**What's needed:**
- [ ] Test with real Axum applications
- [ ] Test with real frontend builds (React, Vue, Svelte)
- [ ] Test development mode with actual dev servers
- [ ] Test production mode with embedded assets
- [ ] Test SPA routing and fallback behavior

### 11. Example Applications
**Status:** Examples exist but may not work

**What's needed:**
- [ ] Verify all examples in `examples/` directory work
- [ ] Test `axum-sveltekit` example specifically
- [ ] Add more framework examples
- [ ] Document example setup steps

## 📝 Documentation

### 12. API Documentation
**Status:** Partial

**What's needed:**
- [ ] Document the actual working API (once implemented)
- [ ] Add troubleshooting guide
- [ ] Document limitations and known issues
- [ ] Add migration guide from other solutions

## Priority Order for Implementation

1. **Tower Service Implementation** - Without this, nothing works
2. **Static File Service** - Needed for production mode
3. **Response Type Compatibility** - Needed for both modes
4. **Proxy Service Enhancements** - Needed for development mode
5. **Integration Tests** - Verify everything works together
6. **Example Verification** - Ensure examples work as documented

## Current Status Summary

**What Works:**
- ✅ Configuration API and builder pattern
- ✅ Route matching and pattern compilation
- ✅ Mode detection (dev vs prod)
- ✅ Error types and error handling structure
- ✅ Basic proxy service structure
- ✅ Basic static file service structure

**What Doesn't Work:**
- ❌ Actually serving static files
- ❌ Actually proxying requests
- ❌ SPA fallback routing
- ❌ File embedding with rust-embed
- ❌ End-to-end request handling

**Estimated Work:**
- Critical items: ~2-3 days of focused development
- Important items: ~1-2 days
- Nice to have: ~1-2 days
- Testing & docs: ~1-2 days

**Total: ~5-10 days** to get to a fully functional v0.1 release
