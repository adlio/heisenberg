use heisenberg::{
    adapters::rocket::{serve_spa, RocketResponse},
    Heisenberg,
};
use rocket::{get, http::Status, launch, routes, serde::json::Json};
use std::path::PathBuf;

#[get("/<path..>")]
async fn spa_handler(path: PathBuf) -> Result<RocketResponse, Status> {
    let config = Heisenberg::new()
        .spa("./web/dist")
        .dev_server("http://localhost:3000")
        .build();
    serve_spa(&path, &config).await
}

#[get("/")]
async fn spa_root() -> Result<RocketResponse, Status> {
    spa_handler(PathBuf::from("index.html")).await
}

#[get("/hello")]
fn api_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "Hello from Rocket API!",
        "framework": "rocket",
        "frontend": "vue"
    }))
}

#[launch]
fn rocket() -> _ {
    println!("🚀 Rocket-Vue example on http://127.0.0.1:8000");
    println!("📦 API: http://127.0.0.1:8000/api/hello");

    rocket::build()
        .mount("/api", routes![api_handler])
        .mount("/", routes![spa_root, spa_handler])
}
