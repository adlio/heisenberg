//! Tests for ProxyService header forwarding and query string handling

use axum::{extract::Query, routing::get, Router};
use heisenberg::{Heisenberg, HeisenbergLayer};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct QueryParams {
    #[serde(flatten)]
    params: HashMap<String, String>,
}

async fn start_echo_server() -> u16 {
    let app = Router::new()
        .route(
            "/echo-headers",
            get(|headers: axum::http::HeaderMap| async move {
                // Return headers as JSON-like string
                let mut result = String::new();
                for (name, value) in headers.iter() {
                    if let Ok(v) = value.to_str() {
                        result.push_str(&format!("{}:{}\n", name, v));
                    }
                }
                result
            }),
        )
        .route(
            "/echo-query",
            get(|Query(params): Query<QueryParams>| async move {
                // Return query params as string
                let mut result: Vec<String> = params
                    .params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                result.sort();
                result.join("&")
            }),
        )
        .route(
            "/echo-both",
            get(
                |headers: axum::http::HeaderMap, Query(params): Query<QueryParams>| async move {
                    let mut lines = Vec::new();

                    // Add query params
                    let mut query_params: Vec<String> = params
                        .params
                        .iter()
                        .map(|(k, v)| format!("query:{}={}", k, v))
                        .collect();
                    query_params.sort();
                    lines.extend(query_params);

                    // Add custom headers (filter for x- prefix)
                    for (name, value) in headers.iter() {
                        if name.as_str().starts_with("x-") {
                            if let Ok(v) = value.to_str() {
                                lines.push(format!("header:{}={}", name, v));
                            }
                        }
                    }

                    lines.join("\n")
                },
            ),
        );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    port
}

#[tokio::test]
async fn test_proxy_forwards_query_string() {
    std::env::set_var("HEISENBERG_MODE", "proxy");
    std::env::set_var("HEISENBERG_SKIP_DEV_SERVER", "1");

    let dev_port = start_echo_server().await;
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
        .get(format!("http://{}/echo-query?foo=bar&baz=qux", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    assert!(body.contains("foo=bar"));
    assert!(body.contains("baz=qux"));

    std::env::remove_var("HEISENBERG_MODE");
    std::env::remove_var("HEISENBERG_SKIP_DEV_SERVER");
}

#[tokio::test]
async fn test_proxy_forwards_custom_headers() {
    std::env::set_var("HEISENBERG_MODE", "proxy");
    std::env::set_var("HEISENBERG_SKIP_DEV_SERVER", "1");

    let dev_port = start_echo_server().await;
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
        .get(format!("http://{}/echo-headers", addr))
        .header("x-custom-header", "custom-value")
        .header("x-another-header", "another-value")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    assert!(body.contains("x-custom-header:custom-value"));
    assert!(body.contains("x-another-header:another-value"));

    std::env::remove_var("HEISENBERG_MODE");
    std::env::remove_var("HEISENBERG_SKIP_DEV_SERVER");
}

#[tokio::test]
async fn test_proxy_preserves_both_headers_and_query() {
    std::env::set_var("HEISENBERG_MODE", "proxy");
    std::env::set_var("HEISENBERG_SKIP_DEV_SERVER", "1");

    let dev_port = start_echo_server().await;
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
        .get(format!("http://{}/echo-both?page=1&limit=10", addr))
        .header("x-request-id", "test-123")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();

    // Check query params
    assert!(
        body.contains("query:page=1"),
        "Expected body to contain 'query:page=1', got: {body}"
    );
    assert!(
        body.contains("query:limit=10"),
        "Expected body to contain 'query:limit=10', got: {body}"
    );

    // Check headers
    assert!(
        body.contains("header:x-request-id=test-123"),
        "Expected body to contain 'header:x-request-id=test-123', got: {body}"
    );

    std::env::remove_var("HEISENBERG_MODE");
    std::env::remove_var("HEISENBERG_SKIP_DEV_SERVER");
}

#[tokio::test]
async fn test_proxy_error_page_contains_troubleshooting() {
    std::env::set_var("HEISENBERG_MODE", "proxy");
    std::env::set_var("HEISENBERG_SKIP_DEV_SERVER", "1");

    let spa = heisenberg::embed_spa!("./tests/fixtures/minimal-spa/dist");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server("http://localhost:59995")
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

    let body = res.text().await.unwrap();
    // Check that the error page contains troubleshooting info
    assert!(body.contains("Development Server Unavailable"));
    assert!(body.contains("Troubleshooting"));
    assert!(body.contains("localhost:59995"));
    assert!(body.contains("npm run dev"));

    std::env::remove_var("HEISENBERG_MODE");
    std::env::remove_var("HEISENBERG_SKIP_DEV_SERVER");
}

#[tokio::test]
async fn test_proxy_forwards_response_headers() {
    std::env::set_var("HEISENBERG_MODE", "proxy");
    std::env::set_var("HEISENBERG_SKIP_DEV_SERVER", "1");

    // Server that sets custom response headers
    let app = Router::new().route(
        "/",
        get(|| async {
            (
                [
                    ("x-custom-response", "response-value"),
                    ("cache-control", "no-cache"),
                ],
                "OK",
            )
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dev_port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let dev_url = format!("http://localhost:{}", dev_port);

    let spa = heisenberg::embed_spa!("./tests/fixtures/minimal-spa/dist");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server(&dev_url)
        .build();

    let proxy_app = Router::new().layer(HeisenbergLayer::new(config));

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = proxy_listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(proxy_listener, proxy_app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{}/", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);

    // Check that response headers were forwarded
    assert_eq!(
        res.headers()
            .get("x-custom-response")
            .map(|v| v.to_str().unwrap()),
        Some("response-value")
    );
    assert_eq!(
        res.headers()
            .get("cache-control")
            .map(|v| v.to_str().unwrap()),
        Some("no-cache")
    );

    std::env::remove_var("HEISENBERG_MODE");
    std::env::remove_var("HEISENBERG_SKIP_DEV_SERVER");
}
