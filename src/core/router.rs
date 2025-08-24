//! Request routing logic for Heisenberg

use crate::core::config::SpaRouteConfig;
use crate::core::mode::Mode;
use crate::error::HeisenbergError;
#[cfg(feature = "logging")]
use tracing::{debug, info, warn};

use std::collections::HashMap;

/// Router for matching requests to SPA routes
#[derive(Debug, Clone)]
pub struct Router {
    routes: Vec<RouteEntry>,
    mode: Mode,
    pattern_cache: HashMap<String, usize>, // Cache for pattern matching
}

/// Internal route entry with compiled matcher
#[derive(Debug, Clone)]
struct RouteEntry {
    matcher: PathMatcher,
    config: SpaRouteConfig,
    #[allow(dead_code)] // Will be used for debugging/logging
    priority: usize, // Lower number = higher priority
}

/// Path matcher for route patterns
#[derive(Debug, Clone)]
enum PathMatcher {
    /// Exact match
    Exact(String),
    /// Prefix match (e.g., "/api/*")
    Prefix(String),
    /// Catch-all match ("/*")
    CatchAll,
}

/// Handler type for a matched route
#[derive(Debug, Clone)]
pub enum RouteHandler {
    /// Proxy to development server
    Proxy(SpaRouteConfig),
    /// Serve static files from embedded assets
    StaticFiles(SpaRouteConfig),
}

impl Router {
    /// Create a new router with the given routes and mode
    pub fn new(routes: Vec<SpaRouteConfig>, mode: Mode) -> Result<Self, HeisenbergError> {
        #[cfg(feature = "logging")]
        info!(
            mode = ?mode,
            route_count = routes.len(),
            "Creating Heisenberg router"
        );

        // Validate routes for conflicts at build time
        Self::validate_routes(&routes)?;

        let mut route_entries = Vec::new();

        // Sort routes by specificity (most specific first)
        let mut sorted_routes: Vec<_> = routes.into_iter().enumerate().collect();
        sorted_routes.sort_by(|(_, a), (_, b)| {
            Self::route_priority(&a.pattern).cmp(&Self::route_priority(&b.pattern))
        });

        for (original_index, route) in sorted_routes {
            let matcher = Self::compile_pattern(&route.pattern)?;
            #[cfg(feature = "logging")]
            debug!(
                pattern = %route.pattern,
                embed_dir = %route.embed_dir.display(),
                dev_proxy_url = %route.dev_proxy_url,
                priority = original_index,
                "Registered route"
            );
            route_entries.push(RouteEntry {
                matcher,
                config: route,
                priority: original_index,
            });
        }

        #[cfg(feature = "logging")]
        info!(
            route_count = route_entries.len(),
            "Router created successfully"
        );

        Ok(Self {
            routes: route_entries,
            mode,
            pattern_cache: HashMap::new(),
        })
    }

    /// Match a request path to a route
    pub fn match_route(&mut self, path: &str) -> Option<&SpaRouteConfig> {
        // Check cache first
        if let Some(&route_index) = self.pattern_cache.get(path) {
            #[cfg(feature = "logging")]
            debug!(path = %path, route_index, "Route match found in cache");
            return self.routes.get(route_index).map(|entry| &entry.config);
        }

        // Find matching route
        for (index, entry) in self.routes.iter().enumerate() {
            if entry.matcher.matches(path) {
                #[cfg(feature = "logging")]
                debug!(
                    path = %path,
                    pattern = %entry.config.pattern,
                    mode = ?self.mode,
                    "Route matched"
                );
                // Cache the result
                self.pattern_cache.insert(path.to_string(), index);
                return Some(&entry.config);
            }
        }

        #[cfg(feature = "logging")]
        warn!(path = %path, "No route matched request");
        None
    }

    /// Get the current mode
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Determine which handler should handle a request path
    pub fn route_handler(&mut self, path: &str) -> Option<RouteHandler> {
        let mode = self.mode;
        if let Some(route_config) = self.match_route(path) {
            match mode {
                Mode::Development => Some(RouteHandler::Proxy(route_config.clone())),
                Mode::Production => Some(RouteHandler::StaticFiles(route_config.clone())),
            }
        } else {
            None
        }
    }

    /// Validate routes for conflicts and contradictory configurations
    fn validate_routes(routes: &[SpaRouteConfig]) -> Result<(), HeisenbergError> {
        // Check for exact duplicate patterns
        let mut seen_patterns = std::collections::HashSet::new();
        for route in routes {
            if !seen_patterns.insert(&route.pattern) {
                return Err(HeisenbergError::config(
                    format!("Duplicate route pattern: {}", route.pattern),
                    "• Each route pattern must be unique\n• Use different patterns like '/app/*' and '/admin/*'\n• Remove duplicate route configurations"
                ));
            }
        }

        // Check for contradictory overlapping patterns
        for (i, route_a) in routes.iter().enumerate() {
            for route_b in routes.iter().skip(i + 1) {
                if Self::patterns_conflict(&route_a.pattern, &route_b.pattern) {
                    return Err(HeisenbergError::config(
                        format!("Conflicting route patterns: '{}' and '{}' - one pattern completely shadows the other", route_a.pattern, route_b.pattern),
                        "• More specific patterns should come first\n• Avoid patterns where one completely contains another\n• Use non-overlapping patterns like '/api/*' and '/app/*'"
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check if two patterns conflict (one completely shadows the other)
    fn patterns_conflict(_pattern_a: &str, _pattern_b: &str) -> bool {
        // Two patterns conflict if one is a strict subset of the other
        // For example: "/admin" and "/admin/*" don't conflict (different specificity)
        // But "/admin/*" and "/admin/*" do conflict (exact duplicates)

        // We already check for exact duplicates above, so this is for future expansion
        // when we add more complex pattern types (regex, etc.)
        false
    }

    /// Compile a route pattern into a matcher
    fn compile_pattern(pattern: &str) -> Result<PathMatcher, HeisenbergError> {
        if pattern.is_empty() {
            return Err(HeisenbergError::config(
                "Route pattern cannot be empty",
                "• Use '/*' to match all paths\n• Use '/app/*' to match paths starting with /app/\n• Patterns must start with '/' and can use '*' wildcards"
            ));
        }

        if pattern == "/*" {
            Ok(PathMatcher::CatchAll)
        } else if let Some(prefix) = pattern.strip_suffix("/*") {
            if prefix.is_empty() {
                Ok(PathMatcher::CatchAll)
            } else {
                Ok(PathMatcher::Prefix(prefix.to_string()))
            }
        } else {
            Ok(PathMatcher::Exact(pattern.to_string()))
        }
    }

    /// Calculate route priority (lower = higher priority)
    fn route_priority(pattern: &str) -> usize {
        if pattern == "/*" {
            // Catch-all has lowest priority
            1000
        } else if let Some(prefix) = pattern.strip_suffix("/*") {
            // Prefix routes: longer prefix = higher priority
            // Use prefix.len() directly
            1000 - prefix.len()
        } else {
            // Exact matches have highest priority
            0
        }
    }
}
impl PathMatcher {
    /// Check if this matcher matches the given path
    fn matches(&self, path: &str) -> bool {
        match self {
            PathMatcher::Exact(exact) => path == exact,
            PathMatcher::Prefix(prefix) => {
                path.starts_with(prefix)
                    && (path.len() == prefix.len() || path.chars().nth(prefix.len()) == Some('/'))
            }
            PathMatcher::CatchAll => true,
        }
    }
}
