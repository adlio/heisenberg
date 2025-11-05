//! Tests for core configuration functionality

use heisenberg::{EmbeddedSpa, Heisenberg};
use std::path::PathBuf;

#[test]
fn test_new_config() {
    let config = Heisenberg::new();
    assert!(config.routes().is_empty());
}

#[test]
fn test_default_config() {
    let config = Heisenberg::default();
    assert!(config.routes().is_empty());
}

#[test]
fn test_single_spa_route() {
    let spa = EmbeddedSpa::new("./dist", "");
    let config = Heisenberg::new().route("/*", spa).build();
    let routes = config.routes();

    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].pattern, "/*");
    assert_eq!(routes[0].embed_dir, PathBuf::from("./dist"));
    assert_eq!(routes[0].dev_proxy_url, "http://localhost:5173");
}

#[test]
fn test_multiple_spa_routes() {
    let spa1 = EmbeddedSpa::new("./admin/dist", "");
    let spa2 = EmbeddedSpa::new("./app/dist", "");
    let config = Heisenberg::new()
        .route("/*", spa1)
        .route("/app/*", spa2)
        .build();

    let routes = config.routes();
    assert_eq!(routes.len(), 2);
    assert_eq!(routes[0].embed_dir, PathBuf::from("./admin/dist"));
    assert_eq!(routes[1].embed_dir, PathBuf::from("./app/dist"));
}

#[test]
fn test_pathbuf_conversion() {
    let spa = EmbeddedSpa::new(PathBuf::from("/absolute/path"), "");
    let config = Heisenberg::new().route("/*", spa).build();
    let routes = config.routes();

    assert_eq!(routes[0].embed_dir, PathBuf::from("/absolute/path"));
}

#[test]
fn test_browser_opening_configuration() {
    let spa = EmbeddedSpa::new("./dist", "");
    let config = Heisenberg::new()
        .route("/*", spa)
        .open_browser(true)
        .build();

    let routes = config.routes();
    assert_eq!(routes.len(), 1);
    assert!(routes[0].open_browser);
}

#[test]
fn test_browser_opening_default_false() {
    let spa = EmbeddedSpa::new("./dist", "");
    let config = Heisenberg::new().route("/*", spa).build();
    let routes = config.routes();

    assert!(!routes[0].open_browser);
}

#[test]
fn test_advanced_configuration() {
    let spa = EmbeddedSpa::new("./dist", "");
    let config = Heisenberg::new()
        .route("/*", spa)
        .working_dir("./frontend")
        .fallback_file("app.html")
        .pattern("/app/*")
        .build();

    let routes = config.routes();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].working_dir, PathBuf::from("./frontend"));
    assert_eq!(routes[0].fallback_file, Some("app.html".to_string()));
    assert_eq!(routes[0].pattern, "/app/*");
}

#[test]
fn test_global_settings() {
    use std::time::Duration;

    let spa = EmbeddedSpa::new("./dist", "");
    let config = Heisenberg::new()
        .health_check_interval(Duration::from_secs(10))
        .proxy_timeout(Duration::from_secs(60))
        .route("/*", spa)
        .build();

    let settings = config.global_settings();
    assert_eq!(settings.health_check_interval, Duration::from_secs(10));
    assert_eq!(settings.proxy_timeout, Duration::from_secs(60));
}

#[test]
fn test_validation_success() {
    let spa = EmbeddedSpa::new("./dist", "");
    let config = Heisenberg::new().route("/*", spa).build();

    assert!(config.validate().is_ok());
}

#[test]
fn test_validation_duplicate_patterns() {
    let spa1 = EmbeddedSpa::new("./dist1", "");
    let spa2 = EmbeddedSpa::new("./dist2", "");
    let config = Heisenberg::new()
        .route("/*", spa1)
        .route("/*", spa2)
        .build();

    assert!(config.validate().is_err());
}
