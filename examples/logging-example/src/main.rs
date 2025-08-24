use axum::{response::Html, routing::get, Router};
use heisenberg::Heisenberg;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,heisenberg=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Configure Heisenberg with logging enabled
    let heisenberg_config = Heisenberg::new()
        .spa("./dist")
        .dev_server("http://localhost:5173")
        .dev_command(["npm", "run", "dev"])
        .open_browser(true)
        .build();

    // Validate configuration (will emit structured logs)
    heisenberg_config.validate().expect("Invalid configuration");

    let app = Router::new()
        .route("/api/hello", get(api_handler))
        .layer(heisenberg_config);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server listening on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn api_handler() -> Html<&'static str> {
    Html("<h1>Hello from API!</h1>")
}
