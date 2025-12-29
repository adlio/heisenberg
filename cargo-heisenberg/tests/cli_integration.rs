//! Integration tests for the cargo-heisenberg CLI binary.
//!
//! These tests verify the CLI argument parsing and command behavior
//! using the actual binary.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn cargo_heisenberg() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("cargo-heisenberg"))
}

#[test]
fn test_help_flag() {
    cargo_heisenberg()
        .arg("heisenberg")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Heisenberg build orchestration"));
}

#[test]
fn test_version_flag() {
    cargo_heisenberg()
        .arg("heisenberg")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cargo-heisenberg"));
}

#[test]
fn test_init_help() {
    cargo_heisenberg()
        .arg("heisenberg")
        .arg("init")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize"));
}

#[test]
fn test_build_help() {
    cargo_heisenberg()
        .arg("heisenberg")
        .arg("build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Build frontend"));
}

#[test]
fn test_run_help() {
    cargo_heisenberg()
        .arg("heisenberg")
        .arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Start frontend"));
}

#[test]
fn test_init_creates_config_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create a web directory with package.json
    let web_dir = temp_dir.path().join("web");
    fs::create_dir_all(&web_dir).unwrap();
    fs::write(web_dir.join("package.json"), "{}").unwrap();

    cargo_heisenberg()
        .arg("heisenberg")
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created heisenberg.toml"));

    // Verify file was created
    let config_path = temp_dir.path().join("heisenberg.toml");
    assert!(config_path.exists());

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[spa]"));
    assert!(content.contains("working_dir"));
}

#[test]
fn test_init_fails_when_config_exists() {
    let temp_dir = TempDir::new().unwrap();

    // Create existing config
    fs::write(temp_dir.path().join("heisenberg.toml"), "existing").unwrap();

    // Create a web directory with package.json
    let web_dir = temp_dir.path().join("web");
    fs::create_dir_all(&web_dir).unwrap();
    fs::write(web_dir.join("package.json"), "{}").unwrap();

    cargo_heisenberg()
        .arg("heisenberg")
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_init_fails_when_no_frontend() {
    let temp_dir = TempDir::new().unwrap();

    cargo_heisenberg()
        .arg("heisenberg")
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Could not find package.json"));
}

#[test]
fn test_unknown_subcommand() {
    cargo_heisenberg()
        .arg("heisenberg")
        .arg("unknown")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
