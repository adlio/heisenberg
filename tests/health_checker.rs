//! Tests for HealthChecker service

use axum::{routing::get, Router};
use heisenberg::services::HealthChecker;
use std::time::Duration;
use tokio::net::TcpListener;

async fn start_mock_server(status_code: u16) -> u16 {
    let app = match status_code {
        200 => Router::new().route("/", get(|| async { "OK" })),
        404 => Router::new(), // No routes = 404
        500 => Router::new().route(
            "/",
            get(|| async { (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Error") }),
        ),
        _ => Router::new(),
    };

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    port
}

#[tokio::test]
async fn test_is_healthy_returns_true_for_200() {
    let port = start_mock_server(200).await;
    let checker = HealthChecker::new(format!("http://localhost:{}", port));

    assert!(checker.is_healthy().await);
}

#[tokio::test]
async fn test_is_healthy_returns_true_for_404() {
    // 404 means the server is running, just no route matched
    let port = start_mock_server(404).await;
    let checker = HealthChecker::new(format!("http://localhost:{}/nonexistent", port));

    assert!(checker.is_healthy().await);
}

#[tokio::test]
async fn test_is_healthy_returns_false_for_connection_refused() {
    // Use a port that's definitely not in use
    let checker = HealthChecker::new("http://localhost:59998".to_string());

    assert!(!checker.is_healthy().await);
}

#[tokio::test]
async fn test_check_health_succeeds_for_200() {
    let port = start_mock_server(200).await;
    let checker = HealthChecker::new(format!("http://localhost:{}", port));

    let result = checker.check_health().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_check_health_succeeds_for_client_error() {
    // Client errors (4xx) are considered healthy - server is responding
    let port = start_mock_server(404).await;
    let checker = HealthChecker::new(format!("http://localhost:{}/missing", port));

    let result = checker.check_health().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_check_health_fails_for_server_error() {
    let port = start_mock_server(500).await;
    let checker = HealthChecker::new(format!("http://localhost:{}", port));

    let result = checker.check_health().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_check_health_fails_for_connection_refused() {
    let checker = HealthChecker::new("http://localhost:59997".to_string());

    let result = checker.check_health().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_wait_for_healthy_succeeds_immediately() {
    let port = start_mock_server(200).await;
    let checker = HealthChecker::new(format!("http://localhost:{}", port));

    let result = checker.wait_for_healthy(Duration::from_secs(5)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_wait_for_healthy_times_out() {
    // Server that doesn't exist
    let checker = HealthChecker::new("http://localhost:59996".to_string());

    let result = checker.wait_for_healthy(Duration::from_millis(500)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_wait_for_healthy_retries_until_success() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    // Server that returns 500 first, then 200
    let app = Router::new().route(
        "/",
        get(move || {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                if count < 2 {
                    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Not ready")
                } else {
                    (axum::http::StatusCode::OK, "Ready")
                }
            }
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let checker = HealthChecker::new(format!("http://localhost:{}", port));
    let result = checker.wait_for_healthy(Duration::from_secs(5)).await;

    assert!(result.is_ok());
    assert!(counter.load(Ordering::SeqCst) >= 3); // At least 3 attempts
}
