//! Tests for error handling

use heisenberg::error::HeisenbergError;
use std::io;

#[test]
fn test_error_display() {
    let config_err = HeisenbergError::config("invalid setting", "Check your configuration");
    assert!(config_err
        .to_string()
        .contains("Configuration error: invalid setting"));
    assert!(config_err.to_string().contains("Troubleshooting:"));
    assert!(config_err.to_string().contains("Check your configuration"));

    let file_err = HeisenbergError::file_not_found("missing.txt", "Check if the file exists");
    assert!(file_err.to_string().contains("File not found: missing.txt"));
    assert!(file_err.to_string().contains("Troubleshooting:"));
    assert!(file_err.to_string().contains("Check if the file exists"));

    let no_route_err = HeisenbergError::NoRouteMatch;
    assert_eq!(no_route_err.to_string(), "No route matched the request");
}

#[test]
fn test_error_from_io() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let heisenberg_err = HeisenbergError::from(io_err);

    match heisenberg_err {
        HeisenbergError::IoError(_) => (),
        _ => panic!("Expected IoError variant"),
    }
}

#[test]
fn test_error_from_json() {
    let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let heisenberg_err = HeisenbergError::from(json_err);

    match heisenberg_err {
        HeisenbergError::PackageJsonParse(_) => (),
        _ => panic!("Expected PackageJsonParse variant"),
    }
}
