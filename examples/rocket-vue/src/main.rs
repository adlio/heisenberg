use rocket::{get, launch, routes, serde::json::Json, fs::NamedFile, response::Redirect, http::Status};
use std::path::{Path, PathBuf};

fn is_dev_mode() -> bool {
    std::env::var("NODE_ENV").unwrap_or_default() != "production"
}

#[get("/<path..>")]
async fn spa_handler(path: PathBuf) -> Result<NamedFile, Status> {
    if is_dev_mode() {
        // In dev mode, proxy to Vue dev server on port 3000
        // For now, return 503 to indicate dev server should be running
        return Err(Status::ServiceUnavailable);
    }
    
    // Production mode: serve static files
    let mut file_path = Path::new("dist").join(&path);
    
    // If path is empty or directory, serve index.html
    if path.as_os_str().is_empty() || file_path.is_dir() {
        file_path = Path::new("dist/index.html").to_path_buf();
    }
    
    NamedFile::open(file_path).await.map_err(|_| Status::NotFound)
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
