//! Tests for the router functionality

use heisenberg::core::config::SpaRouteConfig;
use heisenberg::core::mode::Mode;
use heisenberg::core::router::{RouteHandler, Router};
use std::path::PathBuf;

fn create_test_route(pattern: &str, embed_dir: &str) -> SpaRouteConfig {
    SpaRouteConfig {
        pattern: pattern.to_string(),
        embed_dir: PathBuf::from(embed_dir),
        dev_proxy_url: "http://localhost:3000".to_string(),
        dev_command: vec!["npm".to_string(), "run".to_string(), "dev".to_string()],
        working_dir: PathBuf::from("."),
        fallback_file: Some("index.html".to_string()),
        open_browser: false,
    }
}

#[test]
fn test_router_creation() {
    let routes = vec![create_test_route("/*", "./dist")];

    let router = Router::new(routes, Mode::Development);
    assert!(router.is_ok());
}

#[test]
fn test_exact_route_matching() {
    let routes = vec![
        create_test_route("/admin", "./admin/dist"),
        create_test_route("/app", "./app/dist"),
    ];

    let mut router = Router::new(routes, Mode::Development).unwrap();

    assert!(router.match_route("/admin").is_some());
    assert!(router.match_route("/app").is_some());
    assert!(router.match_route("/other").is_none());
}

#[test]
fn test_prefix_route_matching() {
    let routes = vec![
        create_test_route("/admin/*", "./admin/dist"),
        create_test_route("/api/*", "./api/dist"),
    ];

    let mut router = Router::new(routes, Mode::Development).unwrap();

    assert!(router.match_route("/admin").is_some());
    assert!(router.match_route("/admin/").is_some());
    assert!(router.match_route("/admin/users").is_some());
    assert!(router.match_route("/api/v1/users").is_some());
    assert!(router.match_route("/other").is_none());
}

#[test]
fn test_catch_all_route() {
    let routes = vec![create_test_route("/*", "./dist")];

    let mut router = Router::new(routes, Mode::Development).unwrap();

    assert!(router.match_route("/").is_some());
    assert!(router.match_route("/anything").is_some());
    assert!(router.match_route("/deeply/nested/path").is_some());
}

#[test]
fn test_route_priority() {
    let routes = vec![
        create_test_route("/*", "./dist"), // Catch-all (lowest priority)
        create_test_route("/admin/*", "./admin"), // Prefix (medium priority)
        create_test_route("/admin/users", "./users"), // Exact (highest priority)
    ];

    let mut router = Router::new(routes, Mode::Development).unwrap();

    // Exact match should win over prefix and catch-all
    let matched = router.match_route("/admin/users").unwrap();
    assert_eq!(matched.embed_dir, PathBuf::from("./users"));

    // Prefix should win over catch-all
    let matched = router.match_route("/admin/settings").unwrap();
    assert_eq!(matched.embed_dir, PathBuf::from("./admin"));

    // Catch-all should match everything else
    let matched = router.match_route("/other").unwrap();
    assert_eq!(matched.embed_dir, PathBuf::from("./dist"));
}

#[test]
fn test_route_caching() {
    let routes = vec![create_test_route("/admin/*", "./admin")];

    let mut router = Router::new(routes, Mode::Development).unwrap();

    // First match should populate cache
    assert!(router.match_route("/admin/users").is_some());

    // Second match should use cache (we can't directly test this, but it should work)
    assert!(router.match_route("/admin/users").is_some());
}

#[test]
fn test_invalid_patterns() {
    let routes = vec![SpaRouteConfig {
        pattern: "".to_string(), // Empty pattern should fail
        embed_dir: PathBuf::from("./dist"),
        dev_proxy_url: "http://localhost:3000".to_string(),
        dev_command: vec!["npm".to_string(), "run".to_string(), "dev".to_string()],
        working_dir: PathBuf::from("."),
        fallback_file: Some("index.html".to_string()),
        open_browser: false,
    }];

    let router = Router::new(routes, Mode::Development);
    assert!(router.is_err());
}

#[test]
fn test_route_handler_development_mode() {
    let routes = vec![create_test_route("/admin/*", "./admin/dist")];

    let mut router = Router::new(routes, Mode::Development).unwrap();

    let handler = router.route_handler("/admin/users").unwrap();
    match handler {
        RouteHandler::Proxy(config) => {
            assert_eq!(config.pattern, "/admin/*");
            assert_eq!(config.dev_proxy_url, "http://localhost:3000");
        }
        RouteHandler::StaticFiles(_) => panic!("Expected proxy handler in development mode"),
    }
}

#[test]
fn test_route_handler_production_mode() {
    let routes = vec![create_test_route("/admin/*", "./admin/dist")];

    let mut router = Router::new(routes, Mode::Production).unwrap();

    let handler = router.route_handler("/admin/users").unwrap();
    match handler {
        RouteHandler::StaticFiles(config) => {
            assert_eq!(config.pattern, "/admin/*");
            assert_eq!(config.embed_dir.to_str().unwrap(), "./admin/dist");
        }
        RouteHandler::Proxy(_) => panic!("Expected static files handler in production mode"),
    }
}

#[test]
fn test_route_validation_duplicate_patterns() {
    let routes = vec![
        create_test_route("/admin/*", "./admin1/dist"),
        create_test_route("/admin/*", "./admin2/dist"), // Duplicate pattern
    ];

    let router = Router::new(routes, Mode::Development);
    assert!(router.is_err());

    if let Err(e) = router {
        assert!(e.to_string().contains("Duplicate route pattern"));
    }
}
