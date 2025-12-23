//! Configuration types and builder API

use crate::core::mode::Mode;
#[cfg(feature = "logging")]
use tracing::{debug, info};

use std::path::{Path, PathBuf};
use std::time::Duration;

/// Main configuration builder for Heisenberg
#[derive(Debug, Clone)]
pub struct Heisenberg {
    /// SPA route configurations
    pub routes: Vec<SpaRouteConfig>,
    /// Global settings
    pub global_settings: GlobalSettings,
    /// Mode override (None = auto-detect)
    pub mode_override: Option<Mode>,
}

/// Global settings for Heisenberg
#[derive(Debug, Clone)]
pub struct GlobalSettings {
    /// Health check interval for dev servers
    pub health_check_interval: Duration,
    /// Proxy timeout for requests
    pub proxy_timeout: Duration,
    /// Process startup timeout
    pub process_startup_timeout: Duration,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(5),
            proxy_timeout: Duration::from_secs(30),
            process_startup_timeout: Duration::from_secs(30),
        }
    }
}

/// Configuration for a single SPA route
#[derive(Debug, Clone)]
pub struct SpaRouteConfig {
    /// Route pattern (e.g., "/*", "/admin/*")
    pub pattern: String,
    /// Directory containing embedded assets
    pub embed_dir: PathBuf,
    /// Development proxy URL
    pub dev_proxy_url: String,
    /// Development command to run (e.g., ["npm", "run", "dev"])
    pub dev_command: Vec<String>,
    /// Working directory for the dev command
    pub working_dir: PathBuf,
    /// Fallback file for SPA routing (e.g., "index.html")
    pub fallback_file: Option<String>,
    /// Whether to open browser automatically in development mode
    pub open_browser: bool,
}

impl SpaRouteConfig {
    /// Validate this route configuration
    pub fn validate(&self) -> Result<(), crate::error::HeisenbergError> {
        // Validate pattern
        if self.pattern.is_empty() {
            return Err(crate::error::HeisenbergError::config(
                "Route pattern cannot be empty",
                "• Use '/*' to match all paths\n• Use '/app/*' to match paths starting with /app/\n• Patterns must start with '/' and can use '*' wildcards"
            ));
        }

        // Validate dev command
        if self.dev_command.is_empty() {
            return Err(crate::error::HeisenbergError::config(
                "Development command cannot be empty",
                "• Common commands: ['npm', 'run', 'dev'] or ['yarn', 'dev']\n• Check your package.json scripts section\n• Use .dev_command() to specify a custom command"
            ));
        }

        // Validate dev proxy URL
        if self.dev_proxy_url.is_empty() {
            return Err(crate::error::HeisenbergError::config(
                "Development proxy URL cannot be empty",
                "• Common URLs: 'http://localhost:3000' (React/Next.js)\n• Or: 'http://localhost:5173' (Vite)\n• Use .dev_server() to specify the URL"
            ));
        }

        // Basic URL validation
        if !self.dev_proxy_url.starts_with("http://") && !self.dev_proxy_url.starts_with("https://")
        {
            return Err(crate::error::HeisenbergError::config(
                format!("Development proxy URL must start with http:// or https://: {}", self.dev_proxy_url),
                "• Use 'http://localhost:3000' for local development\n• Use 'https://...' only if your dev server uses HTTPS\n• Check your frontend dev server configuration"
            ));
        }

        Ok(())
    }
}

/// Builder for configuring SPA routes
#[derive(Debug)]
pub struct SpaRouteBuilder {
    heisenberg: Heisenberg,
    route_index: usize,
}

impl SpaRouteBuilder {
    /// Set whether to open browser automatically in development mode
    pub fn open_browser(mut self, open: bool) -> Self {
        if let Some(route) = self.heisenberg.routes.get_mut(self.route_index) {
            route.open_browser = open;
        }
        self
    }

    /// Set the development proxy URL where the frontend dev server runs.
    ///
    /// This is auto-discovered from `heisenberg.toml` or `vite.config.js`,
    /// but can be overridden for advanced use cases.
    pub fn dev_server(mut self, url: &str) -> Self {
        if let Some(route) = self.heisenberg.routes.get_mut(self.route_index) {
            route.dev_proxy_url = url.to_string();
        }
        self
    }

    /// Set the development command to start the frontend dev server.
    ///
    /// This is auto-discovered from `heisenberg.toml` or `package.json`,
    /// but can be overridden for advanced use cases.
    pub fn dev_command<I, S>(mut self, command: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if let Some(route) = self.heisenberg.routes.get_mut(self.route_index) {
            route.dev_command = command
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect();
        }
        self
    }

    /// Set the working directory for the dev command
    pub fn working_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        if let Some(route) = self.heisenberg.routes.get_mut(self.route_index) {
            route.working_dir = dir.into();
        }
        self
    }

    /// Set the fallback file for SPA routing
    pub fn fallback_file<S: Into<String>>(mut self, file: S) -> Self {
        if let Some(route) = self.heisenberg.routes.get_mut(self.route_index) {
            route.fallback_file = Some(file.into());
        }
        self
    }

    /// Set the route pattern
    pub fn pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        if let Some(route) = self.heisenberg.routes.get_mut(self.route_index) {
            route.pattern = pattern.into();
        }
        self
    }

    /// Add another SPA route with pattern and embedded assets
    pub fn route(self, pattern: &str, embedded_spa: crate::EmbeddedSpa) -> SpaRouteBuilder {
        self.heisenberg.route(pattern, embedded_spa)
    }

    /// Finish building and return the Heisenberg config
    pub fn build(self) -> Heisenberg {
        // Note: We don't validate here to keep the API simple
        // Users can call validate() explicitly if needed
        self.heisenberg
    }
}

impl Default for Heisenberg {
    fn default() -> Self {
        Self::new()
    }
}

impl Heisenberg {
    /// Create a new Heisenberg configuration with sensible defaults.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use heisenberg::{EmbeddedSpa, Heisenberg};
    ///
    /// let spa = EmbeddedSpa::new("./dist", "");
    /// let config = Heisenberg::new()
    ///     .route("/*", spa)
    ///     .build();
    /// ```
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            global_settings: GlobalSettings::default(),
            mode_override: None,
        }
    }

    /// Add a SPA route with pattern and embedded assets
    ///
    /// # Arguments
    ///
    /// * `pattern` - Route pattern (e.g., `"/*"`, `"/admin/*"`)
    /// * `embedded_spa` - Handle from `embed_spa!()` macro
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let admin = heisenberg::embed_spa!("./web/admin", admin);
    /// let user = heisenberg::embed_spa!("./web/user", user);
    ///
    /// let config = Heisenberg::new()
    ///     .route("/admin/*", admin)
    ///     .route("/*", user)
    ///     .build();
    /// ```
    pub fn route(mut self, pattern: &str, embedded_spa: crate::EmbeddedSpa) -> SpaRouteBuilder {
        let embed_dir = embedded_spa.build_path();

        // Try to load config from heisenberg.toml, then infer from directory
        let (dev_proxy_url, dev_command, working_dir) = Self::discover_spa_config(&embed_dir);

        let route = SpaRouteConfig {
            pattern: pattern.to_string(),
            embed_dir: embed_dir.clone(),
            dev_proxy_url,
            dev_command,
            working_dir,
            fallback_file: Some("index.html".to_string()),
            open_browser: false,
        };
        self.routes.push(route);
        let route_index = self.routes.len() - 1;

        SpaRouteBuilder {
            heisenberg: self,
            route_index,
        }
    }

    fn discover_spa_config(embed_dir: &Path) -> (String, Vec<String>, PathBuf) {
        // 1. Try to load from heisenberg.toml
        if let Ok(config) = crate::config::HeisenbergConfig::from_file("heisenberg.toml") {
            for spa_config in config.spas() {
                if spa_config.output_dir == embed_dir {
                    let dev_url = spa_config
                        .dev_server
                        .clone()
                        .unwrap_or_else(|| "http://localhost:5173".to_string());
                    let dev_cmd = spa_config
                        .dev_command
                        .clone()
                        .map(|cmd| cmd.split_whitespace().map(String::from).collect())
                        .unwrap_or_else(|| {
                            vec!["npm".to_string(), "run".to_string(), "dev".to_string()]
                        });
                    return (dev_url, dev_cmd, spa_config.working_dir.clone());
                }
            }
        }

        // 2. Try to infer from directory structure
        if let Ok(inferred) = crate::utils::infer_from_build_dir(embed_dir) {
            return (inferred.dev_url, inferred.dev_command, inferred.working_dir);
        }

        // 3. Fallback defaults
        let working_dir = embed_dir.parent().unwrap_or(embed_dir).to_path_buf();
        (
            "http://localhost:5173".to_string(),
            vec!["npm".to_string(), "run".to_string(), "dev".to_string()],
            working_dir,
        )
    }

    /// Get the routes
    pub fn routes(&self) -> &[SpaRouteConfig] {
        &self.routes
    }

    /// Set global health check interval
    pub fn health_check_interval(mut self, interval: Duration) -> Self {
        self.global_settings.health_check_interval = interval;
        self
    }

    /// Set global proxy timeout
    pub fn proxy_timeout(mut self, timeout: Duration) -> Self {
        self.global_settings.proxy_timeout = timeout;
        self
    }

    /// Set global process startup timeout
    pub fn process_startup_timeout(mut self, timeout: Duration) -> Self {
        self.global_settings.process_startup_timeout = timeout;
        self
    }

    /// Override mode detection
    pub fn mode_override(mut self, mode: Mode) -> Self {
        self.mode_override = Some(mode);
        self
    }

    /// Get global settings
    pub fn global_settings(&self) -> &GlobalSettings {
        &self.global_settings
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), crate::error::HeisenbergError> {
        #[cfg(feature = "logging")]
        info!(
            routes = self.routes.len(),
            mode_override = ?self.mode_override,
            "Validating Heisenberg configuration"
        );

        // Check for duplicate patterns
        let mut patterns = std::collections::HashSet::new();
        for route in &self.routes {
            if !patterns.insert(&route.pattern) {
                #[cfg(feature = "logging")]
                debug!(pattern = %route.pattern, "Duplicate route pattern detected");
                return Err(crate::error::HeisenbergError::config(
                    format!("Duplicate route pattern: {}", route.pattern),
                    "• Each route pattern must be unique\n• Use different patterns like '/app/*' and '/admin/*'\n• Remove duplicate .spa() calls with the same pattern"
                ));
            }
        }

        // Validate each route
        for route in &self.routes {
            #[cfg(feature = "logging")]
            debug!(
                pattern = %route.pattern,
                embed_dir = %route.embed_dir.display(),
                dev_proxy_url = %route.dev_proxy_url,
                "Validating route configuration"
            );
            route.validate()?;
        }

        #[cfg(feature = "logging")]
        info!("Configuration validation successful");
        Ok(())
    }
}
