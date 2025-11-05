use chrono::Utc;
use heisenberg::{adapters::rocket::spa_routes, Heisenberg};
use rocket::{get, launch, routes, serde::json::Json};
use serde::Serialize;

#[derive(Serialize)]
struct ApiResponse {
    message: String,
    framework: String,
    frontend: String,
    timestamp: String,
}

#[get("/hello")]
fn api_handler() -> Json<ApiResponse> {
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    Json(ApiResponse {
        message: "Hello from Rocket!".to_string(),
        framework: "rocket".to_string(),
        frontend: "vue".to_string(),
        timestamp,
    })
}

#[launch]
fn rocket() -> _ {
    println!("Rocket-Vue example on http://127.0.0.1:8000");
    println!("API: http://127.0.0.1:8000/api/hello");

    let spa = heisenberg::embed_spa!();
    let config = Heisenberg::new().route("/*", spa).build();

    rocket::build()
        .manage(config)
        .mount("/api", routes![api_handler])
        .mount("/", spa_routes())
}
