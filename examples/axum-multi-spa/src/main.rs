use axum::{extract::Path, response::Json, routing::get, Router};
use heisenberg::{Heisenberg, HeisenbergLayer};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower::ServiceBuilder;

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
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
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
    // Configure Heisenberg with multiple SPA routes
    let heisenberg_config = Heisenberg::new()
        // Admin panel on /admin/* - serves from admin-dist/
        .spa("./admin-dist")
        .pattern("/admin/*")
        .dev_server("http://localhost:3001")
        .dev_command(["npm", "run", "dev:admin"])
        .working_dir("./admin-frontend")
        .fallback_file("index.html")
        .open_browser(false)
        // Main application on /app/* - serves from app-dist/
        .spa("./app-dist")
        .pattern("/app/*")
        .dev_server("http://localhost:3002")
        .dev_command(["npm", "run", "dev:app"])
        .working_dir("./app-frontend")
        .fallback_file("index.html")
        .open_browser(false)
        // Landing page on /* (catch-all) - serves from landing-dist/
        .spa("./landing-dist")
        .pattern("/*")
        .dev_server("http://localhost:3000")
        .dev_command(["npm", "run", "dev"])
        .working_dir("./landing-frontend")
        .fallback_file("index.html")
        .open_browser(true)
        .build(); // Only open browser for main landing page

    // Create Axum app with API routes for different services
    let app = Router::new()
        // Admin API routes
        .route("/api/admin", get(admin_api_handler))
        .route("/api/admin/users/:user_id", get(user_handler))
        // Main app API routes
        .route("/api/app", get(app_api_handler))
        .route("/api/app/users/:user_id", get(user_handler))
        // Global API routes
        .route("/api/status", get(api_status_handler))
        // Add Heisenberg layer for SPA serving
        .layer(ServiceBuilder::new().layer(HeisenbergLayer::new(heisenberg_config)));

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    println!("🚀 Multi-SPA Server running on http://{}", addr);
    println!("📊 Admin Panel: http://{}/admin/", addr);
    println!("📱 Main App: http://{}/app/", addr);
    println!("🏠 Landing Page: http://{}/", addr);
    println!("🔧 API Status: http://{}/api/status", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
