//! WebSocket proxying tests

mod common;

use axum::{routing::get, Router};
use common::EnvGuard;
use futures_util::{SinkExt, StreamExt};
use heisenberg::{Heisenberg, HeisenbergLayer};
use serial_test::serial;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Start an echo server that tracks connection counts
async fn start_echo_server() -> (SocketAddr, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let total_connections = Arc::new(AtomicUsize::new(0));
    let total = total_connections.clone();

    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let total = total.clone();
            tokio::spawn(async move {
                total.fetch_add(1, Ordering::SeqCst);
                if let Ok(ws) = tokio_tungstenite::accept_async(stream).await {
                    let (mut tx, mut rx) = ws.split();
                    while let Some(Ok(msg)) = rx.next().await {
                        if tx.send(msg).await.is_err() {
                            break;
                        }
                    }
                }
            });
        }
    });

    (addr, total_connections)
}

/// Tests WebSocket proxy: basic functionality and that aborted connections
/// don't create orphaned backend connections (the rapid-refresh bug fix).
#[tokio::test]
#[serial]
async fn test_websocket_proxy() {
    let _mode = EnvGuard::set("HEISENBERG_MODE", "proxy");
    let _skip = EnvGuard::set("HEISENBERG_SKIP_DEV_SERVER", "1");

    let (backend_addr, total_connections) = start_echo_server().await;
    let backend_url = format!("http://127.0.0.1:{}", backend_addr.port());

    let spa = heisenberg::EmbeddedSpa::new("./tests/fixtures/minimal-spa/dist", "");
    let config = Heisenberg::new()
        .route("/*", spa)
        .dev_server(&backend_url)
        .dev_command(["echo", "test"])
        .build();

    let app = Router::new()
        .route("/api/test", get(|| async { "API works" }))
        .layer(HeisenbergLayer::new(config));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Part 1: Simulate rapid refresh - aborted upgrades shouldn't create backend connections
    let connections_before = total_connections.load(Ordering::SeqCst);
    let num_attempts = 20;

    for _ in 0..num_attempts {
        use tokio::io::AsyncWriteExt;
        if let Ok(mut stream) = tokio::net::TcpStream::connect(proxy_addr).await {
            let request = format!(
                "GET / HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nUpgrade: websocket\r\n\
                 Connection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
                 Sec-WebSocket-Version: 13\r\n\r\n",
                proxy_addr.port()
            );
            let _ = stream.write_all(request.as_bytes()).await;
            drop(stream); // Abort immediately
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let orphaned = total_connections.load(Ordering::SeqCst) - connections_before;
    assert!(
        orphaned < num_attempts / 2,
        "Too many backend connections ({orphaned}) from {num_attempts} aborted upgrades"
    );

    // Part 2: Verify normal WebSocket still works
    let ws_url = format!("ws://127.0.0.1:{}/", proxy_addr.port());
    let (ws_stream, _) = connect_async(&ws_url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    write.send(Message::Text("Hello".into())).await.unwrap();
    if let Some(Ok(Message::Text(text))) = read.next().await {
        assert_eq!(text, "Hello");
    } else {
        panic!("Expected echo response");
    }
}
