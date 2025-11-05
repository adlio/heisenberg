use axum::{routing::get, Json, Router};
use heisenberg::SpaExt; // Import extension trait
use serde::Serialize;

#[derive(Serialize)]
struct Message {
    text: String,
}

async fn hello() -> Json<Message> {
    Json(Message {
        text: "Hello from Heisenberg!".to_string(),
    })
}

#[tokio::main]
async fn main() {
    // Super simple API - just add .spa() to your router!
    let app = Router::new().route("/api/hello", get(hello)).spa(); // That's it! One line.

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind");

    println!("🚀 Server running on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .with_graceful_shutdown(heisenberg::shutdown_signal())
        .await
        .unwrap();
}
