use heisenberg::{
    adapters::rocket::{serve_spa, RocketResponse},
    Heisenberg,
};
use rocket::{get, http::Status, launch, routes, serde::json::Json, State};
use std::path::PathBuf;

#[get("/<path..>")]
async fn spa_handler(path: PathBuf, config: &State<Heisenberg>) -> Result<RocketResponse, Status> {
    serve_spa(&path, config).await
}

#[get("/")]
async fn spa_root(config: &State<Heisenberg>) -> Result<RocketResponse, Status> {
    serve_spa(&PathBuf::from("index.html"), config).await
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

    let config = Heisenberg::new()
        .spa("./web/dist")
        .dev_server("http://localhost:3000")
        .build();

    rocket::build()
        .manage(config)
        .mount("/api", routes![api_handler])
        .mount("/", routes![spa_root, spa_handler])
}
