//! WebSocket proxying demo with Heisenberg

use axum::{routing::get, Router};
use heisenberg::{Heisenberg, HeisenbergLayer};

#[tokio::main]
async fn main() {
    std::env::set_var("HEISENBERG_MODE", "proxy");

    let config = Heisenberg::new()
        .spa("./dist")
        .dev_server("http://localhost:8080")
        .dev_command(["echo", "Backend should be started separately"])
        .build();

    let app = Router::new()
        .route("/api/health", get(|| async { "OK" }))
        .layer(HeisenbergLayer::new(config));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to 0.0.0.0:3000 - port may already be in use");

    println!("🚀 Heisenberg proxy running on http://localhost:3000");
    println!("📡 Proxying WebSocket connections to ws://localhost:8080");
    println!("\n⚠️  Make sure the backend WebSocket server is running on port 8080");
    println!("   Run: cargo run --bin websocket-backend\n");

    axum::serve(listener, app).await.unwrap();
}
