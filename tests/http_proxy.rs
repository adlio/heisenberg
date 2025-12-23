//! Actual HTTP proxy tests - tests real proxy forwarding

mod common;

use axum::{routing::get, Router};
use common::EnvGuard;
use heisenberg::{Heisenberg, HeisenbergLayer};
use serial_test::serial;
use tokio::net::TcpListener;

async fn start_mock_dev_server() -> u16 {
    let app = Router::new()
        .route("/", get(|| async { "<html>Dev Server</html>" }))
        .route("/api/data", get(|| async { "dev data" }));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    port
}

#[tokio::test]
#[serial]
async fn test_proxy_forwards_requests() {
    let _mode = EnvGuard::set("HEISENBERG_MODE", "proxy");
    let _skip = EnvGuard::set("HEISENBERG_SKIP_DEV_SERVER", "1");

    let dev_port = start_mock_dev_server().await;
    let dev_url = format!("http://localhost:{}", dev_port);

    let spa = heisenberg::embed_spa!("./tests/fixtures/minimal-spa/dist");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server(&dev_url)
        .build();

    let app = Router::new().layer(HeisenbergLayer::new(config));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{}/", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    assert_eq!(body, "<html>Dev Server</html>");
}

#[tokio::test]
#[serial]
async fn test_proxy_skips_api_routes() {
    let _mode = EnvGuard::set("HEISENBERG_MODE", "proxy");
    let _skip = EnvGuard::set("HEISENBERG_SKIP_DEV_SERVER", "1");

    let dev_port = start_mock_dev_server().await;
    let dev_url = format!("http://localhost:{}", dev_port);

    let spa = heisenberg::embed_spa!("./tests/fixtures/minimal-spa/dist");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server(&dev_url)
        .build();

    let app = Router::new()
        .route("/api/test", get(|| async { "backend API" }))
        .layer(HeisenbergLayer::new(config));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{}/api/test", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    assert_eq!(body, "backend API");
}

#[tokio::test]
#[serial]
async fn test_proxy_error_handling() {
    let _mode = EnvGuard::set("HEISENBERG_MODE", "proxy");
    let _skip = EnvGuard::set("HEISENBERG_SKIP_DEV_SERVER", "1");

    let spa = heisenberg::embed_spa!("./tests/fixtures/minimal-spa/dist");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server("http://localhost:59999")
        .build();

    let app = Router::new().layer(HeisenbergLayer::new(config));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{}/", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 503);
}
