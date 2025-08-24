//! Mode detection tests

use heisenberg::core::mode::{detect_mode, Mode};
use std::env;
use std::sync::Mutex;

// Serialize tests that modify environment variables
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_mode_detection_env_override_production() {
    let _guard = ENV_MUTEX.lock().unwrap();

    env::set_var("HEISENBERG_MODE", "production");
    let mode = detect_mode();
    env::remove_var("HEISENBERG_MODE");

    assert_eq!(mode, Mode::Production);
}

#[test]
fn test_mode_detection_env_override_development() {
    let _guard = ENV_MUTEX.lock().unwrap();

    env::set_var("HEISENBERG_MODE", "development");
    let mode = detect_mode();
    env::remove_var("HEISENBERG_MODE");

    assert_eq!(mode, Mode::Development);
}

#[test]
fn test_mode_detection_aliases() {
    let _guard = ENV_MUTEX.lock().unwrap();

    // Test production aliases
    for alias in ["prod", "embed"] {
        env::set_var("HEISENBERG_MODE", alias);
        assert_eq!(
            detect_mode(),
            Mode::Production,
            "Failed for alias: {}",
            alias
        );
    }

    // Test development aliases
    for alias in ["dev", "proxy"] {
        env::set_var("HEISENBERG_MODE", alias);
        assert_eq!(
            detect_mode(),
            Mode::Development,
            "Failed for alias: {}",
            alias
        );
    }

    env::remove_var("HEISENBERG_MODE");
}

#[test]
fn test_mode_detection_default_fallback() {
    let _guard = ENV_MUTEX.lock().unwrap();

    env::remove_var("HEISENBERG_MODE");
    let mode = detect_mode();

    // Should match build configuration
    #[cfg(debug_assertions)]
    assert_eq!(mode, Mode::Development);

    #[cfg(not(debug_assertions))]
    assert_eq!(mode, Mode::Production);
}
