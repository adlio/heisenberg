use axum::{response::Html, routing::get, Router};
use heisenberg::{Heisenberg, HeisenbergLayer};
use std::net::SocketAddr;
use tower::ServiceBuilder;

async fn api_handler() -> Html<&'static str> {
    Html("<h1>API Response from Rust Backend</h1>")
}

#[tokio::main]
async fn main() {
    // Configure Heisenberg
    let heisenberg_config = Heisenberg::new().spa("./dist").build();

    // Create Axum app with API routes
    let app = Router::new()
        .route("/api/hello", get(api_handler))
        .layer(ServiceBuilder::new().layer(HeisenbergLayer::new(heisenberg_config)));

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
