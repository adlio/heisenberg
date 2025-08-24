# Heisenberg Logging Example

This example demonstrates how to use Heisenberg's optional structured logging feature.

## Features Demonstrated

- **Optional logging dependency**: Uses the `logging` feature flag
- **Structured diagnostics**: Shows configuration validation, route matching, and process management
- **Integration with tracing-subscriber**: Standard Rust logging setup
- **Environment-based filtering**: Control log levels via `RUST_LOG`

## Running the Example

```bash
# Enable logging and run
cargo run --features logging

# With custom log levels
RUST_LOG=debug,heisenberg=trace cargo run --features logging

# Without logging (minimal dependencies)
cargo run
```

## Expected Log Output

When running with logging enabled, you'll see structured logs like:

```
INFO heisenberg::core::config: Validating Heisenberg configuration routes=1
DEBUG heisenberg::core::config: Validating route configuration pattern="/*" embed_dir="./dist" dev_proxy_url="http://localhost:5173"
INFO heisenberg::core::config: Configuration validation successful
INFO heisenberg::core::router: Creating Heisenberg router mode=Development route_count=1
DEBUG heisenberg::core::router: Registered route pattern="/*" embed_dir="./dist" dev_proxy_url="http://localhost:5173" priority=0
INFO heisenberg::core::router: Router created successfully
INFO heisenberg::services::process: Starting frontend dev server process command=["npm", "run", "dev"]
DEBUG heisenberg::services::health: Waiting for dev server to become healthy target_url="http://localhost:5173"
INFO heisenberg::services::health: Dev server is healthy target_url="http://localhost:5173"
INFO heisenberg::services::process: Frontend dev server is healthy and ready route_id="/*"
```

## Log Filtering

Use `RUST_LOG` environment variable to control logging:

- `RUST_LOG=info` - Show info level and above
- `RUST_LOG=debug,heisenberg=trace` - Debug for all, trace for Heisenberg
- `RUST_LOG=heisenberg::core::router=debug` - Only router debug logs

## Integration Notes

- Logging is **completely optional** - exclude the feature for minimal dependencies
- Uses standard `tracing` crate for structured logging
- Compatible with any `tracing-subscriber` configuration
- No performance impact when logging feature is disabled