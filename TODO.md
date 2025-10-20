# Heisenberg TODO

Progress tracking for critical functionality gaps.

## ✅ COMPLETED (merged to development branch)

### 1. Tower Service Implementation
- [x] Implement actual routing logic based on Mode
- [x] Call ProxyService in Development mode
- [x] Call StaticFileService in Production mode
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

### 6. Health Checking
**Status:** Implemented but needs verification

**What's needed:**
- [ ] Test with various dev servers (Vite, webpack-dev-server, etc.)
- [ ] Add configurable health check endpoints
- [ ] Handle slow-starting dev servers

### 7. WebSocket Support for HMR
**Status:** Not implemented

**What's needed:**
- [ ] Detect WebSocket upgrade requests
- [ ] Forward WebSocket connections to dev server
- [ ] Handle WebSocket frames bidirectionally

### 8. Process Output Capture
**Status:** Not implemented

**What's needed:**
- [ ] Capture stdout/stderr from dev server processes
- [ ] Log dev server output with tracing
- [ ] Display errors when dev server fails to start

## 📚 Nice to Have

### 9. Configuration Inference
**Status:** Partially implemented

**What's needed:**
- [ ] Test package.json parsing with more frameworks
- [ ] Handle monorepo structures
- [ ] Support more build tools (pnpm, bun, etc.)

### 10. Browser Auto-Open
**Status:** Implemented but needs testing

**What's needed:**
- [ ] Verify works on all platforms (macOS, Linux, Windows)
- [ ] Handle cases where browser can't be opened

## 🧪 Testing Needed

### 11. Integration Tests
- [ ] Test with real Axum applications ✅ (basic test done)
- [ ] Test with real frontend builds ✅ (React, SvelteKit tested)
- [ ] Test development mode with actual dev servers
- [ ] Test SPA routing and fallback behavior ✅ (verified)

### 12. Example Applications
- [x] Verify axum-simple works
- [x] Verify axum-sveltekit works
- [ ] Test other examples (actix-react, rocket-vue)

## 📝 Documentation

### 13. Update Documentation
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
- ✅ Mode detection (dev vs prod)
- ✅ Works with Axum and SvelteKit

**What's Missing:**
- ❌ WebSocket support for HMR
- ❌ Process output capture/logging
- ❌ Comprehensive testing

**Estimated Remaining Work:**
- WebSocket support: ~1 day
- Process output: ~0.5 day
- Testing & docs: ~1 day
- Total: ~2-3 days to v0.2 release
