//! Simple WebSocket echo server backend

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("🎯 WebSocket backend running on ws://localhost:8080");

    while let Ok((stream, addr)) = listener.accept().await {
        println!("📥 New connection from {}", addr);
        tokio::spawn(async move {
            if let Ok(ws) = tokio_tungstenite::accept_async(stream).await {
                let (mut tx, mut rx) = ws.split();
                while let Some(Ok(msg)) = rx.next().await {
                    println!("📨 Received: {:?}", msg);
                    if tx.send(msg).await.is_err() {
                        break;
                    }
                }
            }
            println!("📤 Connection closed: {}", addr);
        });
    }
}
