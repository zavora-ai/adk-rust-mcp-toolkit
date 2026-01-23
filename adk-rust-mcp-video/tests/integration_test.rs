//! Integration tests for adk-rust-mcp-video server.
//!
//! These tests require:
//! - An authenticated Google Cloud SDK (`gcloud auth application-default login`)
//! - PROJECT_ID environment variable set (or uses gcloud default project)
//! - GCS_BUCKET environment variable set (required for video generation)
//! - Vertex AI API enabled in the project
//!
//! Run with: `cargo test --package adk-rust-mcp-video --test integration_test`
//! Skip in CI: `cargo test --package adk-rust-mcp-video --lib`
//!
//! Generated videos are saved to `./test_output/` directory for inspection.

use std::env;
use std::path::PathBuf;
use std::sync::Once;

use adk_rust_mcp_common::config::Config;

static INIT: Once = Once::new();

/// Output directory for test-generated videos
const TEST_OUTPUT_DIR: &str = "test_output";

/// Initialize environment from .env file once
fn init_env() {
    INIT.call_once(|| {
        let _ = dotenvy::dotenv();
    });
}

/// Get or create the test output directory
fn get_test_output_dir() -> PathBuf {
    let dir = PathBuf::from(TEST_OUTPUT_DIR);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create test output directory");
    }
    dir
}

/// Helper to get test configuration from environment.
fn get_test_config() -> Option<Config> {
    init_env();
    
    // Try to get PROJECT_ID from env, or use gcloud default
    let project_id = env::var("PROJECT_ID")
        .or_else(|_| {
            std::process::Command::new("gcloud")
                .args(["config", "get-value", "project"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .ok_or(())
        })
        .ok()?;
    
    Some(Config {
        project_id,
        location: env::var("LOCATION").unwrap_or_else(|_| "us-central1".to_string()),
        gcs_bucket: env::var("GCS_BUCKET").ok(),
        port: 8080,
    })
}

/// Check if integration tests should run.
fn should_run_integration_tests() -> bool {
    if env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return false;
    }
    get_test_config().is_some()
}

/// Macro to skip test if integration tests are disabled.
macro_rules! skip_if_no_integration {
    () => {
        if !should_run_integration_tests() {
            eprintln!("Skipping integration test: no valid configuration");
            return;
        }
    };
}

/// Generate a simple UUID v4 for test uniqueness.
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}

/// Test that the video handler can be created.
#[tokio::test]
async fn test_handler_creation() {
    skip_if_no_integration!();
    
    let config = get_test_config().unwrap();
    let handler = adk_rust_mcp_video::VideoHandler::new(config).await;
    
    assert!(handler.is_ok(), "Handler creation should succeed: {:?}", handler.err());
}

/// Test that validation errors are returned correctly.
#[tokio::test]
async fn test_validation_errors() {
    skip_if_no_integration!();
    
    let config = get_test_config().unwrap();
    let handler = adk_rust_mcp_video::VideoHandler::new(config).await
        .expect("Failed to create handler");
    
    // Invalid duration
    let params = adk_rust_mcp_video::VideoT2vParams {
        prompt: "A cat".to_string(),
        model: "veo-3.0-generate-preview".to_string(),
        aspect_ratio: "16:9".to_string(),
        duration_seconds: 100, // Invalid
        output_gcs_uri: "gs://bucket/output.mp4".to_string(),
        download_local: false,
        local_path: None,
        generate_audio: None,
        seed: None,
    };
    
    let result = handler.generate_video_t2v(params).await;
    assert!(result.is_err(), "Should fail with invalid duration");
}

mod veo_api_tests {
    use super::*;

    /// The current Veo 3 model ID
    const VEO_3_MODEL: &str = "veo-3.0-generate-preview";

    /// Test video generation with text-to-video (GCS output only).
    /// Note: This test is expensive and slow (~2-5 minutes), so it's ignored by default.
    #[tokio::test]
    #[ignore = "Expensive API call - run manually with: cargo test --package adk-rust-mcp-video --test integration_test veo_api_tests::test_video_generation_t2v_gcs -- --ignored"]
    async fn test_video_generation_t2v_gcs() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let gcs_bucket = config.gcs_bucket.clone()
            .expect("GCS_BUCKET must be set for video generation tests");
        
        let handler = adk_rust_mcp_video::VideoHandler::new(config).await
            .expect("Failed to create handler");
        
        let timestamp = uuid_v4();
        let output_uri = format!("gs://{}/integration-test/video_t2v_{}.mp4", gcs_bucket, timestamp);
        
        let params = adk_rust_mcp_video::VideoT2vParams {
            prompt: "A cat walking slowly in a garden, cinematic lighting".to_string(),
            model: VEO_3_MODEL.to_string(),
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 8,
            output_gcs_uri: output_uri.clone(),
            download_local: false,
            local_path: None,
            generate_audio: None,
            seed: Some(42),
        };
        
        eprintln!("Starting video generation (this may take 2-5 minutes)...");
        let result = handler.generate_video_t2v(params).await;
        
        assert!(result.is_ok(), "Video generation should succeed: {:?}", result.err());
        let result = result.unwrap();
        assert!(result.gcs_uri.starts_with("gs://"), "Result should have GCS URI");
        eprintln!("Video generated: {}", result.gcs_uri);
    }

    /// Test video generation with local download.
    /// Note: This test is expensive and slow (~2-5 minutes), so it's ignored by default.
    #[tokio::test]
    #[ignore = "Expensive API call - run manually with: cargo test --package adk-rust-mcp-video --test integration_test veo_api_tests::test_video_generation_t2v_local_download -- --ignored"]
    async fn test_video_generation_t2v_local_download() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let gcs_bucket = config.gcs_bucket.clone()
            .expect("GCS_BUCKET must be set for video generation tests");
        
        let handler = adk_rust_mcp_video::VideoHandler::new(config).await
            .expect("Failed to create handler");
        
        let timestamp = uuid_v4();
        let output_uri = format!("gs://{}/integration-test/video_local_{}.mp4", gcs_bucket, timestamp);
        
        // Use persistent output directory
        let output_dir = get_test_output_dir();
        let local_path = output_dir.join(format!("video_t2v_{}.mp4", timestamp));
        
        let params = adk_rust_mcp_video::VideoT2vParams {
            prompt: "A serene ocean wave rolling onto a beach at sunset".to_string(),
            model: VEO_3_MODEL.to_string(),
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 8,
            output_gcs_uri: output_uri.clone(),
            download_local: true,
            local_path: Some(local_path.to_string_lossy().to_string()),
            generate_audio: None,
            seed: Some(123),
        };
        
        eprintln!("Starting video generation with local download (this may take 2-5 minutes)...");
        let result = handler.generate_video_t2v(params).await;
        
        assert!(result.is_ok(), "Video generation should succeed: {:?}", result.err());
        let result = result.unwrap();
        
        // Verify GCS URI
        assert!(result.gcs_uri.starts_with("gs://"), "Result should have GCS URI");
        eprintln!("Video generated: {}", result.gcs_uri);
        
        // Verify local file
        assert!(result.local_path.is_some(), "Should have local path");
        let local_file = PathBuf::from(result.local_path.as_ref().unwrap());
        assert!(local_file.exists(), "Local file should exist: {}", local_file.display());
        
        let metadata = std::fs::metadata(&local_file).expect("Should read file metadata");
        assert!(metadata.len() > 10000, "Video file should have reasonable size: {} bytes", metadata.len());
        
        eprintln!("Video downloaded to: {} ({} bytes)", local_file.display(), metadata.len());
    }

    /// Test video generation with audio (Veo 3.x only).
    /// Note: This test is expensive and slow (~2-5 minutes), so it's ignored by default.
    #[tokio::test]
    #[ignore = "Expensive API call - run manually"]
    async fn test_video_generation_t2v_with_audio() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let gcs_bucket = config.gcs_bucket.clone()
            .expect("GCS_BUCKET must be set for video generation tests");
        
        let handler = adk_rust_mcp_video::VideoHandler::new(config).await
            .expect("Failed to create handler");
        
        let timestamp = uuid_v4();
        let output_uri = format!("gs://{}/integration-test/video_audio_{}.mp4", gcs_bucket, timestamp);
        
        let params = adk_rust_mcp_video::VideoT2vParams {
            prompt: "A bird singing in a forest with natural ambient sounds".to_string(),
            model: VEO_3_MODEL.to_string(),
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 8,
            output_gcs_uri: output_uri.clone(),
            download_local: false,
            local_path: None,
            generate_audio: Some(true), // Enable audio generation
            seed: Some(456),
        };
        
        eprintln!("Starting video generation with audio (this may take 2-5 minutes)...");
        let result = handler.generate_video_t2v(params).await;
        
        assert!(result.is_ok(), "Video generation with audio should succeed: {:?}", result.err());
        let result = result.unwrap();
        assert!(result.gcs_uri.starts_with("gs://"), "Result should have GCS URI");
        eprintln!("Video with audio generated: {}", result.gcs_uri);
    }
}
