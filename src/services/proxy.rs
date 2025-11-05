//! Proxy service for development mode

use crate::error::HeisenbergError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use tokio_tungstenite::WebSocketStream;

type Body = Full<Bytes>;

/// Proxy service for forwarding requests to dev servers
pub struct ProxyService {
    target_url: String,
    client: reqwest::Client,
}

impl ProxyService {
    /// Create a new proxy service
    pub fn new(target_url: String) -> Self {
        // Configure client for optimal connection pooling
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self { target_url, client }
    }

    /// Proxy a request to the target server
    pub async fn proxy_request(
        &self,
        path: &str,
        query: Option<&str>,
        headers: &hyper::HeaderMap,
    ) -> Result<Response<Body>, HeisenbergError> {
        let target_url = if let Some(q) = query {
            format!("{}{}?{}", self.target_url, path, q)
        } else {
            format!("{}{}", self.target_url, path)
        };

        // Build request with forwarded headers
        let mut req = self.client.get(&target_url);
        for (name, value) in headers.iter() {
            if let Ok(val) = value.to_str() {
                req = req.header(name.as_str(), val);
            }
        }

        match req.send().await {
            Ok(response) => {
                let status = response.status();
                let resp_headers = response.headers().clone();
                let body = response.bytes().await.unwrap_or_default();

                let mut builder = Response::builder().status(status.as_u16());

                // Forward response headers
                for (name, value) in resp_headers.iter() {
                    builder = builder.header(name.as_str(), value.as_bytes());
                }

                Ok(builder.body(Full::new(body)).unwrap())
            }
            Err(e) => Ok(Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .header("content-type", "text/html")
                .body(Full::new(Bytes::from(self.create_error_page(&e))))
                .unwrap()),
        }
    }

    /// Proxy a WebSocket upgrade request
    pub async fn proxy_websocket<B>(
        &self,
        mut req: hyper::Request<B>,
    ) -> Result<Response<Body>, HeisenbergError>
    where
        B: hyper::body::Body + Send + 'static,
        B::Data: Send,
        B::Error: std::error::Error + Send + Sync,
    {
        use hyper_util::rt::TokioIo;
        use tokio_tungstenite::tungstenite::protocol::Role;

        // Extract target URL from request (needed for backend connection later)
        let path = req.uri().path().to_string();
        let query = req
            .uri()
            .query()
            .map(|q| format!("?{}", q))
            .unwrap_or_default();
        let ws_url = self
            .target_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let target = format!("{}{}{}", ws_url, path, query);

        // Get the WebSocket key for the response
        let key = req
            .headers()
            .get("sec-websocket-key")
            .ok_or_else(|| {
                HeisenbergError::proxy("Missing Sec-WebSocket-Key header".to_string(), "")
            })?
            .clone();

        // Spawn upgrade task - connect to backend AFTER client upgrade succeeds
        // This prevents orphaned backend connections when clients disconnect mid-upgrade
        tokio::spawn(async move {
            match hyper::upgrade::on(&mut req).await {
                Ok(upgraded) => {
                    // Client upgrade succeeded - NOW connect to backend
                    match tokio_tungstenite::connect_async(&target).await {
                        Ok((backend_ws, _)) => {
                            let io = TokioIo::new(upgraded);
                            let client_ws =
                                WebSocketStream::from_raw_socket(io, Role::Server, None).await;

                            if let Err(_e) = Self::forward_websocket(client_ws, backend_ws).await {
                                #[cfg(feature = "logging")]
                                tracing::debug!("WebSocket forwarding error: {}", _e);
                            }
                        }
                        Err(_e) => {
                            #[cfg(feature = "logging")]
                            tracing::debug!("Backend WebSocket connection failed: {}", _e);
                            // Client is already upgraded but backend failed - connection will close
                        }
                    }
                }
                Err(_e) => {
                    #[cfg(feature = "logging")]
                    tracing::debug!("WebSocket upgrade failed: {}", _e);
                    // Client disconnected before upgrade completed - no backend connection made
                }
            }
        });

        // Build switching protocols response
        let accept = Self::compute_accept_key(key.as_bytes());
        let response = Response::builder()
            .status(hyper::StatusCode::SWITCHING_PROTOCOLS)
            .header("upgrade", "websocket")
            .header("connection", "Upgrade")
            .header("sec-websocket-accept", accept)
            .body(Full::new(Bytes::new()))
            .unwrap();

        Ok(response)
    }

    /// Compute WebSocket accept key
    fn compute_accept_key(key: &[u8]) -> String {
        use base64::{engine::general_purpose, Engine as _};
        use sha1::{Digest, Sha1};
        const WS_GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

        let mut hasher = Sha1::new();
        hasher.update(key);
        hasher.update(WS_GUID);
        general_purpose::STANDARD.encode(hasher.finalize())
    }

    /// Forward messages between client and backend WebSockets
    async fn forward_websocket<T>(
        client: WebSocketStream<T>,
        backend: WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        use futures_util::{SinkExt, StreamExt};

        let (mut client_tx, mut client_rx) = client.split();
        let (mut backend_tx, mut backend_rx) = backend.split();

        let client_to_backend = async {
            while let Some(msg) = client_rx.next().await {
                if let Ok(msg) = msg {
                    if backend_tx.send(msg).await.is_err() {
                        break;
                    }
                }
            }
        };

        let backend_to_client = async {
            while let Some(msg) = backend_rx.next().await {
                if let Ok(msg) = msg {
                    if client_tx.send(msg).await.is_err() {
                        break;
                    }
                }
            }
        };

        tokio::select! {
            _ = client_to_backend => {},
            _ = backend_to_client => {},
        }

        Ok(())
    }

    /// Create an enhanced error page for dev server unavailability
    fn create_error_page(&self, error: &reqwest::Error) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Development Server Unavailable</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; }}
        .container {{ max-width: 600px; margin: 0 auto; }}
        .error {{ background: #fee; border: 1px solid #fcc; padding: 20px; border-radius: 8px; }}
        .info {{ background: #eff; border: 1px solid #cdf; padding: 20px; border-radius: 8px; margin-top: 20px; }}
        code {{ background: #f5f5f5; padding: 2px 4px; border-radius: 3px; }}
        ul {{ margin: 10px 0; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="error">
            <h1>&#x1F6AB; Development Server Unavailable</h1>
            <p><strong>Could not connect to:</strong> <code>{}</code></p>
            <p><strong>Error:</strong> {}</p>
        </div>
        
        <div class="info">
            <h2>&#x1F4A1; Troubleshooting</h2>
            <p>Heisenberg is trying to proxy to <code>{}</code> but the server is not responding.</p>
            <ul>
                <li><strong>Check if the dev server is running:</strong> Look for a process on this port</li>
                <li><strong>Port mismatch?</strong> If your vite.config.js uses a dynamic port (variables, expressions), 
                    Heisenberg can only detect literal numbers. Use <code>.dev_server(\"http://localhost:PORT\")</code> to specify manually.</li>
                <li><strong>Start the dev server manually:</strong> Run <code>npm run dev</code> in your frontend directory</li>
                <li><strong>Check for port conflicts:</strong> Another process might be using the port</li>
                <li><strong>Wait a moment:</strong> The dev server might still be starting up</li>
            </ul>
            <p><em>This page will automatically work once the development server is available.</em></p>
        </div>
    </div>
    
    <script>
        // Auto-refresh every 5 seconds to check if dev server is back
        setTimeout(() => {{ window.location.reload(); }}, 5000);
    </script>
</body>
</html>"#,
            self.target_url, error, self.target_url
        )
    }
}
