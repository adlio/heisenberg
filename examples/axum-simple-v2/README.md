# Axum Simple V2 - New Ergonomic API

This example showcases the improved developer experience with Heisenberg's new extension trait API.

## Key Improvements

### Before (verbose):
```rust
let app = heisenberg::embed_spa!("./dist");
let config = heisenberg::Heisenberg::new().route("/*", app).build();

let app = Router::new()
    .route("/api/hello", get(hello))
    .layer(heisenberg::HeisenbergLayer::new(config));
```

### After (one-liner):
```rust
use heisenberg::SpaExt;

let app = Router::new()
    .route("/api/hello", get(hello))
    .spa("./dist");  // Just works!
```

## Features Demonstrated

- ✅ Single-line SPA integration
- ✅ Built-in graceful shutdown
- ✅ Clean, readable code
- ✅ No boilerplate

## Running

```bash
cargo run
```
