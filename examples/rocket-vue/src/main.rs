use heisenberg::{
    adapters::rocket::{serve_spa_uri, RocketResponse},
    Heisenberg,
};
use rocket::{
    get,
    http::Status,
    launch,
    request::{self, FromRequest, Request},
    routes,
    serde::json::Json,
    State,
};

// Request guard to capture full URI with query string
pub struct FullUri(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for FullUri {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let path = req.uri().path().as_str();
        let full = if let Some(query) = req.uri().query() {
            format!("{}?{}", path, query.as_str())
        } else {
            path.to_string()
        };
        request::Outcome::Success(FullUri(full))
    }
}

#[get("/<_..>", rank = 2)]
async fn spa_handler(uri: FullUri, config: &State<Heisenberg>) -> Result<RocketResponse, Status> {
    serve_spa_uri(&uri.0, config).await
}

#[get("/", rank = 1)]
async fn spa_root(config: &State<Heisenberg>) -> Result<RocketResponse, Status> {
    serve_spa_uri("index.html", config).await
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
