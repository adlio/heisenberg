//! Mode detection tests

use heisenberg::core::mode::{detect_mode, Mode};
use std::env;
use std::sync::Mutex;

// Serialize tests that modify environment variables
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_mode_detection_env_override_embed() {
    let _guard = ENV_MUTEX.lock().unwrap();

    env::set_var("HEISENBERG_MODE", "embed");
    let mode = detect_mode();
    env::remove_var("HEISENBERG_MODE");

    assert_eq!(mode, Mode::Embed);
}

#[test]
fn test_mode_detection_env_override_proxy() {
    let _guard = ENV_MUTEX.lock().unwrap();

    env::set_var("HEISENBERG_MODE", "proxy");
    let mode = detect_mode();
    env::remove_var("HEISENBERG_MODE");

    assert_eq!(mode, Mode::Proxy);
}

#[test]
fn test_mode_detection_no_aliases() {
    let _guard = ENV_MUTEX.lock().unwrap();

    // Only "embed" and "proxy" are valid
    env::set_var("HEISENBERG_MODE", "embed");
    assert_eq!(detect_mode(), Mode::Embed);

    env::set_var("HEISENBERG_MODE", "proxy");
    assert_eq!(detect_mode(), Mode::Proxy);

    // Invalid values fall back to default (Embed)
    env::set_var("HEISENBERG_MODE", "invalid");
    assert_eq!(detect_mode(), Mode::Embed);

    env::set_var("HEISENBERG_MODE", "dev");
    assert_eq!(detect_mode(), Mode::Embed);

    env::remove_var("HEISENBERG_MODE");
}

#[test]
fn test_mode_detection_default_fallback() {
    let _guard = ENV_MUTEX.lock().unwrap();

    env::remove_var("HEISENBERG_MODE");
    let mode = detect_mode();

    // Default is always Embed mode
    assert_eq!(mode, Mode::Embed);
}
