//! Tests for core configuration functionality

use heisenberg::Heisenberg;
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
    let config = Heisenberg::new().spa("./dist").build();
    let routes = config.routes();

    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].pattern, "/*");
    assert_eq!(routes[0].embed_dir, PathBuf::from("./dist"));
    assert_eq!(routes[0].dev_proxy_url, "http://localhost:5173");
}

#[test]
fn test_multiple_spa_routes() {
    let config = Heisenberg::new()
        .spa("./admin/dist")
        .spa("./app/dist")
        .build();

    let routes = config.routes();
    assert_eq!(routes.len(), 2);
    assert_eq!(routes[0].embed_dir, PathBuf::from("./admin/dist"));
    assert_eq!(routes[1].embed_dir, PathBuf::from("./app/dist"));
}

#[test]
fn test_pathbuf_conversion() {
    let config = Heisenberg::new()
        .spa(PathBuf::from("/absolute/path"))
        .build();
    let routes = config.routes();

    assert_eq!(routes[0].embed_dir, PathBuf::from("/absolute/path"));
}

#[test]
fn test_browser_opening_configuration() {
    let config = Heisenberg::new()
        .spa("./dist")
        .open_browser(true)
        .dev_server("http://localhost:3000")
        .build();

    let routes = config.routes();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].open_browser, true);
    assert_eq!(routes[0].dev_proxy_url, "http://localhost:3000");
}

#[test]
fn test_browser_opening_default_false() {
    let config = Heisenberg::new().spa("./dist").build();
    let routes = config.routes();

    assert_eq!(routes[0].open_browser, false); // Conservative default
}

#[test]
fn test_advanced_configuration() {
    let config = Heisenberg::new()
        .spa("./dist")
        .dev_command(["yarn", "dev"])
        .working_dir("./frontend")
        .fallback_file("app.html")
        .pattern("/app/*")
        .build();

    let routes = config.routes();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].dev_command, vec!["yarn", "dev"]);
    assert_eq!(routes[0].working_dir, PathBuf::from("./frontend"));
    assert_eq!(routes[0].fallback_file, Some("app.html".to_string()));
    assert_eq!(routes[0].pattern, "/app/*");
}

#[test]
fn test_global_settings() {
    use std::time::Duration;

    let config = Heisenberg::new()
        .health_check_interval(Duration::from_secs(10))
        .proxy_timeout(Duration::from_secs(60))
        .spa("./dist")
        .build();

    let settings = config.global_settings();
    assert_eq!(settings.health_check_interval, Duration::from_secs(10));
    assert_eq!(settings.proxy_timeout, Duration::from_secs(60));
}

#[test]
fn test_validation_success() {
    let config = Heisenberg::new().spa("./dist").build();

    assert!(config.validate().is_ok());
}

#[test]
fn test_validation_duplicate_patterns() {
    let config = Heisenberg::new()
        .spa("./dist1")
        .pattern("/*")
        .spa("./dist2")
        .pattern("/*")
        .build();

    assert!(config.validate().is_err());
}
