//! Integration tests for embed_spa! macro
//!
//! embed_spa! has two modes:
//! - Path mode: embed_spa!("./path") - infers config from scanning the directory
//! - Name mode: embed_spa!("name") - looks up in heisenberg.toml

// =============================================================================
// Path Mode Tests - Direct dist directory
// =============================================================================

#[test]
fn test_embed_spa_path_mode_direct_dist() {
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

// =============================================================================
// Path Mode Tests - Node.js project with auto-detection
// =============================================================================

#[test]
fn test_embed_spa_path_mode_with_nodejs_project_dist() {
    // sample_spa has package.json + dist folder
    let spa = heisenberg::embed_spa!("./tests/fixtures/sample_spa");
    // Should auto-detect dist directory
    assert!(
        spa.spa_dir().to_string_lossy().ends_with("dist"),
        "Expected spa_dir to end with 'dist', got: {:?}",
        spa.spa_dir()
    );
}

#[test]
fn test_embed_spa_path_mode_react_app_build() {
    // react-app has package.json + build folder (Create React App style)
    let spa = heisenberg::embed_spa!("./tests/fixtures/react-app");
    // Should auto-detect build directory (has higher priority than dist)
    assert!(
        spa.spa_dir().to_string_lossy().ends_with("build"),
        "Expected spa_dir to end with 'build', got: {:?}",
        spa.spa_dir()
    );
}

#[test]
fn test_embed_spa_path_mode_vite_app_dist() {
    // vite-app has package.json + dist folder
    let spa = heisenberg::embed_spa!("./tests/fixtures/vite-app");
    assert!(
        spa.spa_dir().to_string_lossy().ends_with("dist"),
        "Expected spa_dir to end with 'dist', got: {:?}",
        spa.spa_dir()
    );
}

#[test]
fn test_embed_spa_path_mode_nextjs_app() {
    // nextjs-app has package.json + .next folder
    let spa = heisenberg::embed_spa!("./tests/fixtures/nextjs-app");
    assert!(
        spa.spa_dir().to_string_lossy().ends_with(".next"),
        "Expected spa_dir to end with '.next', got: {:?}",
        spa.spa_dir()
    );
}

#[test]
fn test_embed_spa_path_mode_sveltekit_app() {
    // sveltekit-app has package.json + .svelte-kit/output folder
    let spa = heisenberg::embed_spa!("./tests/fixtures/sveltekit-app");
    assert!(
        spa.spa_dir()
            .to_string_lossy()
            .ends_with(".svelte-kit/output"),
        "Expected spa_dir to end with '.svelte-kit/output', got: {:?}",
        spa.spa_dir()
    );
}

#[test]
fn test_embed_spa_path_mode_node_app_only_dist() {
    // empty-node-app has package.json + only dist folder (no build/.next/.svelte-kit)
    // Tests that dist is correctly selected when it's the only option
    let spa = heisenberg::embed_spa!("./tests/fixtures/empty-node-app");
    assert!(
        spa.spa_dir().to_string_lossy().ends_with("dist"),
        "Expected spa_dir to end with 'dist', got: {:?}",
        spa.spa_dir()
    );
}

// =============================================================================
// Multiple SPAs in same application
// =============================================================================

#[test]
fn test_multiple_embedded_spas() {
    let spa1 = heisenberg::embed_spa!("./tests/fixtures/minimal-spa/dist");
    let spa2 = heisenberg::embed_spa!("./tests/fixtures/sample_spa");

    // Each should be distinct
    assert_ne!(
        spa1.spa_dir().to_string_lossy(),
        spa2.spa_dir().to_string_lossy()
    );

    // Both should work in config
    let config = heisenberg::Heisenberg::new()
        .route("/app/*", spa1)
        .route("/admin/*", spa2)
        .build();

    assert_eq!(config.routes().len(), 2);
}
