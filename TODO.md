# Heisenberg TODO

Actionable tasks for future development sessions.

## ✅ COMPLETED (v0.2.0)

### Core Features
- [x] Tower Service Implementation with routing logic
- [x] Static File Service with SPA fallback
- [x] Process Management with auto-start
- [x] Proxy Service with headers and query strings
- [x] **WebSocket Proxying** - Full HMR support for Vite, Next.js, CRA
- [x] **Blocking Dev Server Startup** - Prevents race conditions
- [x] **Port Auto-Detection** - Reads from vite.config.js and package.json
- [x] **Graceful Shutdown** - SIGINT handler for dev server cleanup
- [x] Mode detection (proxy vs embed)
- [x] Health checking for dev servers
- [x] Browser auto-open
- [x] Process output capture (stdout/stderr inherit)

### Testing & Examples
- [x] WebSocket proxy tests
- [x] Integration tests for all core features
- [x] Working examples: axum-simple, axum-sveltekit, axum-multi-spa
- [x] Logging example
- [x] WebSocket demo example

### Documentation
- [x] Complete README with examples
- [x] API documentation
- [x] CHANGELOG for v0.2.0
- [x] Clear error messages across all examples

## 🎯 High Priority (v0.3.0)

### Framework Examples & Testing
**Goal:** Verify all examples work correctly across platforms

- [ ] **Test actix-react example**
  - Run on macOS, verify dev server starts
  - Test proxy mode and embed mode
  - Document any issues found
  
- [ ] **Test rocket-vue example**
  - Run on macOS, verify dev server starts
  - Test proxy mode and embed mode
  - Document any issues found

- [ ] **Add Warp example**
  - Create examples/warp-react with basic setup
  - Copy pattern from axum-simple
  - Verify Tower integration works

### Cross-Platform Testing
**Goal:** Ensure Heisenberg works on all major platforms

- [ ] **Windows testing**
  - Test axum-sveltekit example on Windows
  - Verify path handling (backslashes vs forward slashes)
  - Test process spawning (npm.cmd vs npm)
  - Document Windows-specific issues

- [ ] **Linux testing**
  - Test axum-sveltekit example on Linux
  - Verify browser auto-open works
  - Test signal handling (SIGINT)

### Build Tool Support
**Goal:** Support modern JavaScript tooling

- [ ] **pnpm support**
  - Detect pnpm-lock.yaml
  - Use `pnpm run dev` instead of `npm run dev`
  - Test with example project

- [ ] **bun support**
  - Detect bun.lockb
  - Use `bun run dev` instead of `npm run dev`
  - Test with example project

## 🔧 Medium Priority

### Developer Experience Improvements

- [ ] **Configurable health check timeout**
  - Add `.health_check_timeout(Duration)` to SpaRouteBuilder
  - Default: 30s, allow override for slow-starting servers
  - Update examples to show usage

- [ ] **Better dev server output formatting**
  - Prefix dev server output with colored labels
  - Example: `[vite] VITE v7.1.3 ready in 590 ms`
  - Make it easy to distinguish dev server logs from app logs

- [ ] **Dev server restart on crash**
  - Detect when dev server process exits unexpectedly
  - Automatically restart with exponential backoff
  - Log restart attempts clearly

### Monorepo Support
**Goal:** Work seamlessly in monorepo structures

- [ ] **Detect workspace root**
  - Look for workspace indicators (pnpm-workspace.yaml, lerna.json)
  - Search up directory tree for package.json with workspaces
  - Use correct working directory for dev commands

- [ ] **Support workspace-relative paths**
  - Allow `./packages/frontend` style paths
  - Resolve relative to workspace root
  - Document monorepo setup patterns

## 📚 Low Priority

### Advanced Features (if demand exists)

- [ ] **Custom proxy middleware**
  - Add `.proxy_middleware(fn)` to intercept/modify requests
  - Use case: Add auth headers, transform responses
  - Design API first, implement if requested

- [ ] **Request/response transformation hooks**
  - Add `.on_request(fn)` and `.on_response(fn)` hooks
  - Use case: Logging, metrics, debugging
  - Design API first, implement if requested

### Performance

- [ ] **Performance benchmarks**
  - Create benches/proxy_throughput.rs
  - Measure requests/second in proxy mode
  - Compare to direct dev server access
  - Document results in README

- [ ] **Load testing**
  - Use `wrk` or `hey` to stress test proxy mode
  - Identify bottlenecks
  - Optimize hot paths if needed

## 📊 Known Limitations

### Accepted Trade-offs
- Dev servers may be orphaned on CTRL-C (acceptable for development)
- Dynamic port configuration (variables in vite.config.js) requires manual `.dev_server()` override
- Tests must run with `--test-threads=1` due to OnceLock state sharing

### Potential Future Improvements (if issues arise)
- Better cleanup of orphaned processes (OS-specific solutions)
- JavaScript AST parser for complex vite.config.js (only if users request it)
- Parallel test execution support (requires refactoring OnceLock pattern)

## 📝 Notes

- v0.2.0 released 2025-11-02 with WebSocket support and blocking startup
- All critical features for production use are implemented
- Focus: Polish existing features, expand framework support, improve cross-platform reliability
- Philosophy: Automatic detection > Configuration files. Keep API simple and fluent.
