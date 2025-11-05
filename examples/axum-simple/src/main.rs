use axum::{response::Html, routing::get, Router};
use heisenberg::SpaExt;
use std::net::SocketAddr;

async fn api_handler() -> Html<&'static str> {
    Html("<h1>API Response from Rust Backend</h1>")
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/hello", get(api_handler)).spa(); // One line!

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", addr));

    axum::serve(listener, app)
        .with_graceful_shutdown(heisenberg::shutdown_signal())
        .await
        .unwrap();
}
