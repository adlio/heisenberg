//! Proxy service for development mode

use crate::error::HeisenbergError;
use crate::services::health::HealthChecker;
use hyper::{Response, StatusCode};
use std::sync::Arc;

/// Proxy service for forwarding requests to dev servers
pub struct ProxyService {
    target_url: String,
    client: reqwest::Client,
    health_checker: Arc<HealthChecker>,
}

impl ProxyService {
    /// Create a new proxy service
    pub fn new(target_url: String) -> Self {
        let health_checker = Arc::new(HealthChecker::new(target_url.clone()));

        // Configure client for optimal connection pooling
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            target_url,
            client,
            health_checker,
        }
    }

    /// Proxy a request to the target server
    pub async fn proxy_request(&self, path: &str) -> Result<Response<String>, HeisenbergError> {
        // Quick health check before proxying
        if !self.health_checker.is_healthy().await {
            return Ok(Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .header("content-type", "text/html")
                .body(self.create_unavailable_error_page())
                .unwrap());
        }

        let target_url = format!("{}{}", self.target_url, path);

        match self.client.get(&target_url).send().await {
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();

                Ok(Response::builder()
                    .status(status.as_u16())
                    .header("content-type", "text/html")
                    .body(body)
                    .unwrap())
            }
            Err(e) => {
                // Return enhanced error page when dev server unavailable
                Ok(Response::builder()
                    .status(StatusCode::SERVICE_UNAVAILABLE)
                    .header("content-type", "text/html")
                    .body(self.create_error_page(&e))
                    .unwrap())
            }
        }
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
            <h1>🚫 Development Server Unavailable</h1>
            <p><strong>Could not connect to:</strong> <code>{}</code></p>
            <p><strong>Error:</strong> {}</p>
        </div>
        
        <div class="info">
            <h2>💡 Troubleshooting</h2>
            <p>The frontend development server is not responding. Here's what you can try:</p>
            <ul>
                <li><strong>Check if the dev server is running:</strong> Look for a process running on the configured port</li>
                <li><strong>Verify the URL:</strong> Make sure <code>{}</code> is correct</li>
                <li><strong>Start the dev server manually:</strong> Run <code>npm run dev</code> or <code>yarn dev</code> in your frontend directory</li>
                <li><strong>Check for port conflicts:</strong> Another process might be using the same port</li>
                <li><strong>Wait a moment:</strong> The dev server might still be starting up</li>
            </ul>
            <p><em>This page will automatically work once the development server is available.</em></p>
        </div>
    </div>
    
    <script>
        // Auto-refresh every 3 seconds to check if dev server is back
        setTimeout(() => {{ window.location.reload(); }}, 3000);
    </script>
</body>
</html>"#,
            self.target_url, error, self.target_url
        )
    }

    /// Create error page for when health check fails
    fn create_unavailable_error_page(&self) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Development Server Unavailable</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; }}
        .container {{ max-width: 600px; margin: 0 auto; }}
        .error {{ background: #fee; border: 1px solid #fcc; padding: 20px; border-radius: 8px; }}
        code {{ background: #f5f5f5; padding: 2px 4px; border-radius: 3px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="error">
            <h1>⏳ Development Server Starting</h1>
            <p>The development server at <code>{}</code> is not ready yet.</p>
            <p><em>This page will refresh automatically...</em></p>
        </div>
    </div>
    
    <script>
        setTimeout(() => {{ window.location.reload(); }}, 2000);
    </script>
</body>
</html>"#,
            self.target_url
        )
    }
}
