//! Actual HTTP serving tests - tests real request/response cycles

mod common;

use axum::Router;
use common::EnvGuard;
use heisenberg::{Heisenberg, HeisenbergLayer};
use serial_test::serial;
use tokio::net::TcpListener;

#[tokio::test]
#[serial]
async fn test_serve_index_html_embed_mode() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");

    let spa = heisenberg::embed_spa!("./tests/fixtures/minimal-spa/dist");
    let config = Heisenberg::new().route("/*", spa).build();

    let app = Router::new().layer(HeisenbergLayer::new(config));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{}/", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    assert!(body.contains("Hello from Heisenberg!"));
    assert!(body.contains("<!DOCTYPE html>"));
}

#[tokio::test]
#[serial]
async fn test_serve_static_asset() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");

    let spa = heisenberg::embed_spa!("./tests/fixtures/minimal-spa/dist");
    let config = Heisenberg::new().route("/*", spa).build();

    let app = Router::new().layer(HeisenbergLayer::new(config));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{}/app.js", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    assert_eq!(
        res.headers().get("content-type").unwrap(),
        "application/javascript; charset=utf-8"
    );
    let body = res.text().await.unwrap();
    assert!(body.contains("Heisenberg static file serving works!"));
}

#[tokio::test]
#[serial]
async fn test_spa_fallback_for_nested_routes() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");

    let spa = heisenberg::embed_spa!("./tests/fixtures/minimal-spa/dist");
    let config = Heisenberg::new().route("/*", spa).build();

    let app = Router::new().layer(HeisenbergLayer::new(config));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{}/some/nested/route", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    assert!(body.contains("Hello from Heisenberg!"));
}

#[tokio::test]
#[serial]
async fn test_404_for_missing_asset() {
    let _guard = EnvGuard::remove("HEISENBERG_MODE");

    let spa = heisenberg::embed_spa!("./tests/fixtures/minimal-spa/dist");
    let config = Heisenberg::new().route("/assets/*", spa).build();

    let app = Router::new().layer(HeisenbergLayer::new(config));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{}/nonexistent.js", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}
