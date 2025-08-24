//! Frontend process management

use crate::error::HeisenbergError;
use crate::services::health::HealthChecker;
use crate::utils::open_browser;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
#[cfg(feature = "logging")]
use tracing::{info, warn};

/// Process manager for frontend dev servers
pub struct ProcessManager {
    processes: Arc<Mutex<HashMap<String, ProcessHandle>>>,
}

/// Handle for a managed process
struct ProcessHandle {
    child: Child,
    startup_time: Instant,
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start a frontend dev server process
    pub async fn start_process(
        &self,
        route_id: &str,
        command: &[String],
        working_dir: &std::path::Path,
        dev_server_url: &str,
        open_browser_flag: bool,
    ) -> Result<(), HeisenbergError> {
        #[cfg(feature = "logging")]
        info!(
            command = ?command,
            working_dir = %working_dir.display(),
            dev_server_url = %dev_server_url,
            open_browser = open_browser_flag,
            "Starting frontend dev server process"
        );

        if command.is_empty() {
            return Err(HeisenbergError::process(
                "Empty command provided",
                "• Specify a dev command like ['npm', 'run', 'dev']\n• Check your package.json scripts section\n• Use .dev_command() to set a custom command"
            ));
        }

        let mut cmd = Command::new(&command[0]);
        cmd.args(&command[1..])
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn().map_err(|e| {
            HeisenbergError::process(
                format!("Failed to start process '{}': {}", command.join(" "), e),
                "• Ensure the command exists (npm, yarn, pnpm)\n• Check if package.json exists in the working directory\n• Verify the working directory path is correct\n• Try running the command manually to test it"
            )
        })?;

        let handle = ProcessHandle {
            child,
            startup_time: Instant::now(),
        };

        self.processes
            .lock()
            .unwrap()
            .insert(route_id.to_string(), handle);

        // Wait for the dev server to become healthy
        let health_checker = HealthChecker::new(dev_server_url.to_string());
        health_checker
            .wait_for_healthy(Duration::from_secs(30))
            .await?;

        #[cfg(feature = "logging")]
        info!(
            route_id = %route_id,
            dev_server_url = %dev_server_url,
            "Frontend dev server is healthy and ready"
        );

        // Open browser if requested
        if open_browser_flag {
            if let Err(e) = open_browser(dev_server_url).await {
                #[cfg(feature = "logging")]
                warn!(error = %e, "Failed to open browser");
                #[cfg(not(feature = "logging"))]
                eprintln!("Warning: Failed to open browser: {}", e);
            }
        }

        Ok(())
    }

    /// Check if a process is running
    pub fn is_process_running(&self, route_id: &str) -> bool {
        let mut processes = self.processes.lock().unwrap();

        if let Some(handle) = processes.get_mut(route_id) {
            // Check if process is still alive
            match handle.child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited, remove it
                    processes.remove(route_id);
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(_) => {
                    // Error checking status, assume dead
                    processes.remove(route_id);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Stop a specific process
    pub fn stop_process(&self, route_id: &str) -> Result<(), HeisenbergError> {
        let mut processes = self.processes.lock().unwrap();

        if let Some(mut handle) = processes.remove(route_id) {
            let _ = handle.child.kill();
            let _ = handle.child.wait();
        }

        Ok(())
    }

    /// Stop all managed processes
    pub fn stop_all_processes(&self) -> Result<(), HeisenbergError> {
        let mut processes = self.processes.lock().unwrap();

        for (_, mut handle) in processes.drain() {
            let _ = handle.child.kill();
            let _ = handle.child.wait();
        }

        Ok(())
    }

    /// Get process uptime
    pub fn get_process_uptime(&self, route_id: &str) -> Option<Duration> {
        let processes = self.processes.lock().unwrap();

        processes
            .get(route_id)
            .map(|handle| handle.startup_time.elapsed())
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        let _ = self.stop_all_processes();
    }
}
