//! Integration tests for adk-rust-mcp-education server.
//!
//! Run with: `cargo test --package adk-rust-mcp-education --test integration_test`

use std::env;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_env() {
    INIT.call_once(|| { let _ = dotenvy::dotenv(); });
}

fn should_run() -> bool {
    if env::var("SKIP_INTEGRATION_TESTS").is_ok() { return false; }
    init_env();
    env::var("GEMINI_API_KEY").is_ok()
}

#[tokio::test]
async fn test_server_creation() {
    use adk_rust_mcp_common::Config;
    use adk_rust_mcp_education::EducationServer;

    init_env();
    if let Ok(config) = Config::from_env() {
        let _server = EducationServer::new(config);
    }
}
