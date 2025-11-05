use chrono::Utc;
use heisenberg::{adapters::rocket::spa_routes, embed_spa, Heisenberg};
use rocket::{get, launch, routes, serde::json::Json};

#[get("/hello")]
fn api_handler() -> Json<serde_json::Value> {
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    Json(serde_json::json!({
        "message": "Hello from API!",
        "framework": "rocket",
        "timestamp": timestamp
    }))
}

#[launch]
fn rocket() -> _ {
    println!("🚀 Multi-SPA example on http://127.0.0.1:8000");
    println!("📦 Main app: http://127.0.0.1:8000/");
    println!("📦 Admin app: http://127.0.0.1:8000/admin/");
    println!("📦 API: http://127.0.0.1:8000/api/hello");

    let admin = embed_spa!("admin");
    let app = embed_spa!("app");

    let config = Heisenberg::new()
        .route("/admin/*", admin)
        .route("/*", app)
        .build();

    rocket::build()
        .manage(config)
        .mount("/api", routes![api_handler])
        .mount("/", spa_routes())
}
