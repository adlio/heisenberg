use axum::{extract::Path, response::Json, routing::get, Router};
use heisenberg::SpaExt;
use serde_json::{json, Value};
use std::net::SocketAddr;

// API handlers for different services
async fn admin_api_handler() -> Json<Value> {
    Json(json!({
        "service": "admin",
        "message": "Admin API endpoint",
        "features": ["user_management", "system_config", "analytics"]
    }))
}

async fn app_api_handler() -> Json<Value> {
    Json(json!({
        "service": "app",
        "message": "Main application API endpoint",
        "features": ["dashboard", "profile", "notifications"]
    }))
}

async fn api_status_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "services": {
            "admin": "running",
            "app": "running"
        }
    }))
}

async fn user_handler(Path(user_id): Path<String>) -> Json<Value> {
    Json(json!({
        "user_id": user_id,
        "name": format!("User {}", user_id),
        "role": if user_id == "1" { "admin" } else { "user" }
    }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        // Admin API routes
        .route("/api/admin", get(admin_api_handler))
        .route("/api/admin/users/:user_id", get(user_handler))
        // Main app API routes
        .route("/api/app", get(app_api_handler))
        .route("/api/app/users/:user_id", get(user_handler))
        // Global API routes
        .route("/api/status", get(api_status_handler))
        // Multiple SPAs - clean and simple!
        .spa_at_from("/admin/*", "./admin-dist")
        .spa_at_from("/app/*", "./app-dist")
        .spa_at_from("/*", "./landing-dist");

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    println!("🚀 Multi-SPA Server running on http://{}", addr);
    println!("📊 Admin Panel: http://{}/admin/", addr);
    println!("📱 Main App: http://{}/app/", addr);
    println!("🏠 Landing Page: http://{}/", addr);
    println!("🔧 API Status: http://{}/api/status", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", addr));

    axum::serve(listener, app)
        .with_graceful_shutdown(heisenberg::shutdown_signal())
        .await
        .unwrap();
}
