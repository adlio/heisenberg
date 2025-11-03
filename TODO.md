# Heisenberg TODO

Progress tracking for future enhancements.

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

## 🔧 Future Enhancements

### Framework Support
- [ ] Test actix-react example thoroughly
- [ ] Test rocket-vue example thoroughly
- [ ] Add Warp example
- [ ] Document framework-specific patterns

### Configuration
- [ ] Support monorepo structures
- [ ] Support more build tools (pnpm, bun, deno)
- [ ] Add configuration file support (heisenberg.toml)
- [ ] Environment-specific configuration

### Developer Experience
- [ ] Better dev server output formatting
- [ ] Configurable health check endpoints
- [ ] Configurable health check timeout
- [ ] Dev server restart on crash
- [ ] Hot reload configuration changes

### Advanced Features
- [ ] Multiple dev servers per SPA (e.g., API + frontend)
- [ ] Custom proxy middleware
- [ ] Request/response transformation hooks
- [ ] Metrics and monitoring
- [ ] Rate limiting for dev mode

### Testing
- [ ] Performance benchmarks
- [ ] Load testing for proxy mode
- [ ] Cross-platform testing (Windows, Linux, macOS)
- [ ] Integration tests with real frontend frameworks

### Documentation
- [ ] Video tutorials
- [ ] Migration guides from other solutions
- [ ] Troubleshooting guide
- [ ] Best practices guide
- [ ] Architecture documentation

## 📊 Known Limitations

### Accepted Trade-offs
- Dev servers may be orphaned on CTRL-C (acceptable for development)
- Dynamic port configuration (variables in vite.config.js) requires manual `.dev_server()` override
- Tests must run with `--test-threads=1` due to OnceLock state sharing

### Future Improvements
- Better cleanup of orphaned processes
- JavaScript AST parser for complex vite.config.js (if demand exists)
- Parallel test execution support

## 🎯 Next Release (v0.3.0)

Potential focus areas:
1. **Stability**: Comprehensive cross-platform testing
2. **DX**: Better error messages and debugging tools
3. **Examples**: More framework examples and real-world patterns
4. **Performance**: Benchmarks and optimizations

## 📝 Notes

- v0.2.0 released 2025-11-02 with WebSocket support and blocking startup
- All critical features for production use are implemented
- Focus shifting to polish, testing, and additional framework support
