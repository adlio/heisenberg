//! WebSocket proxying tests

use axum::{routing::get, Router};
use futures_util::{SinkExt, StreamExt};
use heisenberg::{Heisenberg, HeisenbergLayer};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Start a simple WebSocket echo server for testing
async fn start_echo_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
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

    addr
}

#[tokio::test]
async fn test_websocket_proxy_basic() {
    // Start backend echo server
    let backend_addr = start_echo_server().await;
    let backend_url = format!("http://127.0.0.1:{}", backend_addr.port());

    // Create Heisenberg config pointing to backend
    std::env::set_var("HEISENBERG_MODE", "proxy");
    let config = Heisenberg::new()
        .spa("./test-dist")
        .dev_server(&backend_url)
        .dev_command(["echo", "test"])
        .build();

    // Create Axum app with Heisenberg
    let app = Router::new()
        .route("/api/test", get(|| async { "API works" }))
        .layer(HeisenbergLayer::new(config));

    // Start test server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Connect WebSocket client to proxy
    let ws_url = format!("ws://127.0.0.1:{}/", proxy_addr.port());
    let (ws_stream, _) = connect_async(&ws_url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    // Send test message
    write.send(Message::Text("Hello".into())).await.unwrap();

    // Receive echo
    if let Some(Ok(Message::Text(text))) = read.next().await {
        assert_eq!(text, "Hello");
    } else {
        panic!("Expected text message");
    }

    std::env::remove_var("HEISENBERG_MODE");
}
