//! Basic integration tests for Heisenberg

mod common;

use common::TestSpaFixture;
use heisenberg::Heisenberg;

#[test]
fn test_basic_configuration() {
    let fixture = TestSpaFixture::new().expect("Failed to create test fixture");

    let config = Heisenberg::new().spa(fixture.dist_path()).build();

    assert_eq!(config.routes().len(), 1);
    assert_eq!(config.routes()[0].pattern, "/*");
    assert_eq!(config.routes()[0].embed_dir, *fixture.dist_path());
    assert_eq!(config.routes()[0].dev_proxy_url, "http://localhost:5173");
}

#[test]
fn test_multiple_spa_routes() {
    let fixture1 = TestSpaFixture::new().expect("Failed to create test fixture 1");
    let fixture2 = TestSpaFixture::new().expect("Failed to create test fixture 2");

    let config = Heisenberg::new()
        .spa(fixture1.dist_path())
        .spa(fixture2.dist_path())
        .build();

    assert_eq!(config.routes().len(), 2);
    assert_eq!(config.routes()[0].embed_dir, *fixture1.dist_path());
    assert_eq!(config.routes()[1].embed_dir, *fixture2.dist_path());
}
