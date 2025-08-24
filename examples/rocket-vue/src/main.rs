use heisenberg::{
    adapters::rocket::{serve_spa, RocketResponse},
    Heisenberg,
};
use rocket::{get, launch, routes, serde::json::Json};
use std::path::PathBuf;

#[get("/<path..>")]
async fn spa_handler(path: PathBuf) -> Result<RocketResponse, rocket::http::Status> {
    let config = Heisenberg::new().spa("./dist").build();
    serve_spa(&path, &config).await
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
    println!("Starting Rocket-Vue example on http://127.0.0.1:8000");

    rocket::build()
        .mount("/api", routes![api_handler])
        .mount("/", routes![spa_handler])
}
