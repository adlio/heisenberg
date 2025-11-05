use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use heisenberg::Heisenberg;
use serde::Serialize;

#[derive(Serialize)]
struct ApiResponse {
    message: String,
    framework: String,
    frontend: String,
    timestamp: String,
}

async fn api_hello() -> impl Responder {
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    HttpResponse::Ok().json(ApiResponse {
        message: "Hello from Actix!".to_string(),
        framework: "actix-web".to_string(),
        frontend: "react".to_string(),
        timestamp,
    })
}

async fn spa_handler(
    req: HttpRequest,
    config: web::Data<Heisenberg>,
) -> actix_web::Result<HttpResponse> {
    heisenberg::adapters::actix::serve_spa(&req, &config).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Actix-React example on http://127.0.0.1:8080");
    println!("📦 API: http://127.0.0.1:8080/api/hello");

    let spa = heisenberg::embed_spa!("./frontend");
    let config = web::Data::new(Heisenberg::new().route("/*", spa).build());

    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .route("/api/hello", web::get().to(api_hello))
            .default_service(web::to(spa_handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
