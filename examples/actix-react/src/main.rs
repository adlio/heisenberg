use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};
use heisenberg::{adapters::actix::serve_spa, Heisenberg};

async fn spa_handler(req: HttpRequest) -> Result<HttpResponse> {
    let config = Heisenberg::new().spa("./dist").build();
    serve_spa(&req, &config).await
}

async fn api_handler() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Hello from Actix-web API!",
        "framework": "actix-web",
        "frontend": "react"
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Actix-React example on http://127.0.0.1:8080");

    HttpServer::new(|| {
        App::new()
            .route("/api/hello", web::get().to(api_handler))
            .route("/", web::get().to(spa_handler))
            .route("/{path:.*}", web::get().to(spa_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
