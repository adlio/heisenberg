use axum::{response::Json, routing::get, Router};
use chrono::Utc;
use heisenberg::{tower::HeisenbergLayer, Heisenberg};
use serde_json::json;

async fn api_handler() -> Json<serde_json::Value> {
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    Json(json!({
        "message": "Hello from API!",
        "framework": "axum",
        "timestamp": timestamp
    }))
}

#[tokio::main]
async fn main() {
    println!("🚀 Multi-SPA example on http://127.0.0.1:3002");
    println!("📦 Main app: http://127.0.0.1:3002/");
    println!("📦 Admin app: http://127.0.0.1:3002/admin/");
    println!("📦 API: http://127.0.0.1:3002/api/hello\n");

    let admin = heisenberg::embed_spa!("admin");
    let user = heisenberg::embed_spa!("user");

    let config = Heisenberg::new()
        .route("/admin/*", admin)
        .route("/*", user)
        .build();

    let app = Router::new()
        .route("/api/hello", get(api_handler))
        .layer(HeisenbergLayer::new(config));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3002")
        .await
        .unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(heisenberg::shutdown_signal())
        .await
        .unwrap();
}
