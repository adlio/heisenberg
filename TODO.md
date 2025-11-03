# Heisenberg TODO

Progress tracking for critical functionality gaps.

## ✅ COMPLETED

### 1. Tower Service Implementation
- [x] Implement actual routing logic based on Mode
- [x] Call ProxyService in Proxy mode
- [x] Call StaticFileService in Embed mode
- [x] Handle response type conversions (Body types)
- [x] Implement SPA fallback logic

### 2. Static File Service
- [x] Read files from filesystem
- [x] Path traversal protection
- [x] MIME type detection
- [x] SPA fallback to index.html for 404s
- [x] Handle binary files (images, fonts, etc.)

### 3. Response Type Compatibility
- [x] Update services to return Response<Body>
- [x] Generic over body types for framework compatibility
- [x] Properly convert between hyper body types

### 4. Process Management
- [x] Integrate ProcessManager into tower service
- [x] Auto-start dev servers in development mode
- [x] Process lifecycle management (start, stop, cleanup)

### 5. Proxy Service Enhancements
- [x] Forward request headers properly
- [x] Preserve response headers from dev server
- [x] Support query strings
- [x] Handle binary responses (use bytes() not text())

## 🔧 Important - Remaining Work

### 6. Startup Order Fix
**Status:** In Progress

**Problem:** Dev servers start in background while Rust server binds port, causing race conditions and confusing errors.

**Solution:** Block in HeisenbergLayer::new() to start dev servers synchronously before any framework binds ports.

**Implementation Plan:**
1. Move dev server startup from async spawn to blocking call in layer constructor
2. Use `tokio::runtime::Handle::current().block_on()` to wait for:
   - Dev server process start
   - Health check confirmation
3. Only return from constructor once dev servers are ready
4. Works for ALL frameworks (Axum, Actix, Rocket) - same API

**Benefits:**
- ✅ Dev servers always start before Rust server binds port
- ✅ Clear error messages if dev server fails to start
- ✅ No race conditions or port conflicts
- ✅ Same simple API for all frameworks
- ✅ Fail-fast behavior in proxy mode

**Tasks:**
- [ ] Refactor tower/service.rs to block in constructor
- [ ] Test with Axum example
- [ ] Verify error messages are clear
- [ ] Update documentation

### 7. Health Checking
**Status:** Implemented but needs verification

**What's needed:**
- [ ] Test with various dev servers (Vite, webpack-dev-server, etc.)
- [ ] Add configurable health check endpoints
- [ ] Handle slow-starting dev servers

### 8. WebSocket Support for HMR
**Status:** ✅ Implemented

**Completed:**
- [x] Detect WebSocket upgrade requests
- [x] Forward WebSocket connections to dev server
- [x] Handle WebSocket frames bidirectionally
- [x] Automated testing with echo server
- [x] Working in axum-sveltekit example

### 9. Process Output Capture
**Status:** Not implemented

**What's needed:**
- [ ] Capture stdout/stderr from dev server processes
- [ ] Log dev server output with tracing
- [ ] Display errors when dev server fails to start

## 📚 Nice to Have

### 10. Configuration Inference
**Status:** Partially implemented

**What's needed:**
- [ ] Test package.json parsing with more frameworks
- [ ] Handle monorepo structures
- [ ] Support more build tools (pnpm, bun, etc.)

### 11. Browser Auto-Open
**Status:** Implemented but needs testing

**What's needed:**
- [ ] Verify works on all platforms (macOS, Linux, Windows)
- [ ] Handle cases where browser can't be opened

## 🧪 Testing Needed

### 12. Integration Tests
- [ ] Test with real Axum applications ✅ (basic test done)
- [ ] Test with real frontend builds ✅ (React, SvelteKit tested)
- [ ] Test development mode with actual dev servers
- [ ] Test SPA routing and fallback behavior ✅ (verified)

### 13. Example Applications
- [x] Verify axum-simple works
- [x] Verify axum-sveltekit works
- [ ] Test other examples (actix-react, rocket-vue)

## 📝 Documentation

### 14. Update Documentation
- [ ] Document actual working API
- [ ] Add troubleshooting guide
- [ ] Document layer ordering for API routes
- [ ] Add migration examples

## Summary

**What Works:**
- ✅ Static file serving from filesystem
- ✅ Proxy to dev servers with headers
- ✅ SPA fallback routing
- ✅ Auto-start dev servers
- ✅ Mode detection (proxy vs embed)
- ✅ Works with Axum and SvelteKit
- ✅ WebSocket support for HMR
- ✅ Process output capture (stdout/stderr inherit)

**What's Missing:**
- ❌ Comprehensive testing across all frameworks
- ❌ More example applications

**Estimated Remaining Work:**
- Testing & docs: ~1 day
- Additional examples: ~1 day
- Total: ~2 days to v0.2 release
