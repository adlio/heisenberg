//! Health checking for development servers

use crate::error::HeisenbergError;
use std::time::Duration;
use tokio::time::timeout;
#[cfg(feature = "logging")]
use tracing::{debug, info};

/// Health checker for development servers
pub struct HealthChecker {
    target_url: String,
    client: reqwest::Client,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(target_url: String) -> Self {
        Self {
            target_url,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
        }
    }

    /// Check if the target server is healthy
    pub async fn is_healthy(&self) -> bool {
        self.check_health().await.is_ok()
    }

    /// Perform a health check
    pub async fn check_health(&self) -> Result<(), HeisenbergError> {
        let health_check = async {
            let response = self.client.get(&self.target_url).send().await?;

            if response.status().is_success() || response.status().is_client_error() {
                // Any response (even 404) means the server is running
                Ok(())
            } else {
                Err(HeisenbergError::health_check(
                    format!("Health check failed with status: {}", response.status()),
                    "• The dev server is running but returned an error\n• Check the dev server logs for issues\n• Verify the dev server URL is correct\n• Some servers return 404 for the root path - this is usually fine"
                ))
            }
        };

        // Timeout the health check
        timeout(Duration::from_secs(3), health_check)
            .await
            .map_err(|_| HeisenbergError::health_check(
                "Health check timed out",
                "• The dev server may be starting up slowly\n• Check if the dev server process is running\n• Verify the dev server URL and port\n• Some servers take longer to start - consider increasing timeout"
            ))?
    }

    /// Wait for the server to become healthy
    pub async fn wait_for_healthy(&self, max_wait: Duration) -> Result<(), HeisenbergError> {
        #[cfg(feature = "logging")]
        debug!(
            target_url = %self.target_url,
            max_wait = ?max_wait,
            "Waiting for dev server to become healthy"
        );

        let start = std::time::Instant::now();

        while start.elapsed() < max_wait {
            if self.is_healthy().await {
                #[cfg(feature = "logging")]
                info!(
                    target_url = %self.target_url,
                    elapsed = ?start.elapsed(),
                    "Dev server is healthy"
                );
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Err(HeisenbergError::health_check(
            format!("Server did not become healthy within {:?}", max_wait),
            "• The dev server is taking too long to start\n• Check if the dev command is correct\n• Verify dependencies are installed (npm install)\n• Look at the process logs for startup errors\n• Some servers need more time - try increasing the timeout"
        ))
    }
}
