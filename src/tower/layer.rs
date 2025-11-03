//! Tower layer implementation

use crate::core::config::Heisenberg;
use crate::core::mode::{detect_mode, Mode};
use crate::services::process::ProcessManager;
use crate::tower::service::HeisenbergService;
use std::sync::Arc;
use tower_layer::Layer;
#[cfg(feature = "logging")]
use tracing::debug;

/// Tower layer for Heisenberg dual-mode serving
#[derive(Clone)]
pub struct HeisenbergLayer {
    config: Heisenberg,
    process_manager: Option<Arc<ProcessManager>>,
}

impl std::fmt::Debug for HeisenbergLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeisenbergLayer")
            .field("config", &self.config)
            .field(
                "process_manager",
                &self.process_manager.as_ref().map(|_| "ProcessManager"),
            )
            .finish()
    }
}

impl HeisenbergLayer {
    /// Create a new Heisenberg layer
    ///
    /// In proxy mode, this starts dev servers synchronously before returning.
    /// This ensures dev servers are ready before the Rust server binds its port.
    pub fn new(config: Heisenberg) -> Self {
        let mode = detect_mode();

        let process_manager = if mode == Mode::Proxy
            && std::env::var("HEISENBERG_SKIP_DEV_SERVER").is_err()
        {
            use std::sync::OnceLock;
            static PM: OnceLock<Arc<ProcessManager>> = OnceLock::new();

            let pm = PM.get_or_init(|| {
                let pm = Arc::new(ProcessManager::new());

                // Register signal handler for cleanup (only if runtime available)
                if tokio::runtime::Handle::try_current().is_ok() {
                    let pm_cleanup = pm.clone();
                    tokio::spawn(async move {
                        let _ = tokio::signal::ctrl_c().await;
                        #[cfg(feature = "logging")]
                        debug!("Received SIGINT, cleaning up dev servers");
                        let _ = pm_cleanup.stop_all_processes();
                    });
                }

                // Start dev servers - use separate runtime to avoid nested runtime issues
                let pm_clone = pm.clone();
                let routes = config.routes().to_vec();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        for route in routes {
                            if let Err(e) = pm_clone
                                .start_process(
                                    &route.pattern,
                                    &route.dev_command,
                                    &route.working_dir,
                                    &route.dev_proxy_url,
                                    route.open_browser,
                                )
                                .await
                            {
                                eprintln!("❌ Failed to start dev server: {}", e);
                                eprintln!("   This will cause proxy requests to fail.");
                                eprintln!("   Check the error above and fix the configuration.");
                                std::process::exit(1);
                            }
                        }
                    });
                })
                .join()
                .unwrap();

                pm
            });

            Some(pm.clone())
        } else {
            None
        };

        Self {
            config,
            process_manager,
        }
    }
}

impl<S> Layer<S> for HeisenbergLayer {
    type Service = HeisenbergService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HeisenbergService::new(inner, self.config.clone(), self.process_manager.clone())
            .expect("Failed to create HeisenbergService with router")
    }
}
