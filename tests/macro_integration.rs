//! Integration tests for embed_spa! macro
//!
//! embed_spa! has two modes:
//! - Path mode: embed_spa!("./path") - infers config from scanning the directory
//! - Name mode: embed_spa!("name") - looks up in heisenberg.toml

#[test]
fn test_embed_spa_path_mode() {
    // Path mode: contains "/" so it scans the directory
    let spa = heisenberg::embed_spa!("./tests/fixtures/minimal-spa/dist");
    assert_eq!(spa.build_subdir(), "");
    assert!(spa.spa_dir().to_string_lossy().ends_with("dist"));
}

#[test]
fn test_embed_spa_in_config_builder() {
    let spa = heisenberg::embed_spa!("./tests/fixtures/minimal-spa/dist");
    let config = heisenberg::Heisenberg::new().route("/*", spa).build();

    assert_eq!(config.routes().len(), 1);
    assert_eq!(config.routes()[0].pattern, "/*");
}
