//! Integration tests for adk-rust-mcp-music server.
//!
//! Run with: `cargo test --package adk-rust-mcp-music --test integration_test`
//! Skip in CI: `cargo test --package adk-rust-mcp-music --lib`
//!
//! These tests require:
//! - Valid Google Cloud credentials (ADC)
//! - PROJECT_ID environment variable set
//! - Access to Vertex AI Lyria API

use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_music::handler::{MusicGenerateParams, MusicHandler};
use std::env;
use std::path::PathBuf;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize environment from .env file once
fn init_env() {
    INIT.call_once(|| {
        let _ = dotenvy::dotenv();
    });
}

/// Helper to get test configuration from environment.
fn get_test_config() -> Option<Config> {
    init_env();
    
    let project_id = env::var("PROJECT_ID").ok()?;
    
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

const TEST_OUTPUT_DIR: &str = "test_output";

fn get_test_output_dir() -> PathBuf {
    let dir = PathBuf::from(TEST_OUTPUT_DIR);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create test output directory");
    }
    dir
}

#[tokio::test]
async fn test_handler_creation() {
    skip_if_no_integration!();
    
    let config = get_test_config().unwrap();
    let handler = MusicHandler::new(config).await;
    assert!(handler.is_ok(), "Failed to create handler: {:?}", handler.err());
}

#[tokio::test]
async fn test_validation_empty_prompt() {
    let params = MusicGenerateParams {
        prompt: "".to_string(),
        negative_prompt: None,
        seed: None,
        sample_count: 1,
        output_file: None,
        output_gcs_uri: None,
    };

    let result = params.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "prompt"));
}

#[tokio::test]
async fn test_validation_invalid_sample_count() {
    let params = MusicGenerateParams {
        prompt: "A jazz tune".to_string(),
        negative_prompt: None,
        seed: None,
        sample_count: 5, // Invalid: max is 4
        output_file: None,
        output_gcs_uri: None,
    };

    let result = params.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "sample_count"));
}

#[tokio::test]
async fn test_validation_valid_params() {
    let params = MusicGenerateParams {
        prompt: "A relaxing piano melody".to_string(),
        negative_prompt: Some("drums".to_string()),
        seed: Some(42),
        sample_count: 2,
        output_file: None,
        output_gcs_uri: None,
    };

    assert!(params.validate().is_ok());
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

mod lyria_api_tests {
    use super::*;
    use adk_rust_mcp_music::handler::MusicGenerateResult;

    /// Test music generation returning base64 data.
    /// Note: This test is expensive and slow, so it's ignored by default.
    #[tokio::test]
    #[ignore = "Expensive API call - run manually with: cargo test --package adk-rust-mcp-music --test integration_test lyria_api_tests::test_music_generation_base64 -- --ignored"]
    async fn test_music_generation_base64() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let handler = MusicHandler::new(config).await.expect("Failed to create handler");
        
        let params = MusicGenerateParams {
            prompt: "A short upbeat electronic melody with synth sounds".to_string(),
            negative_prompt: Some("vocals, drums".to_string()),
            seed: Some(12345),
            sample_count: 1,
            output_file: None,
            output_gcs_uri: None,
        };
        
        eprintln!("Starting music generation (this may take a while)...");
        let result = handler.generate_music(params).await;
        
        match result {
            Ok(MusicGenerateResult::Base64(samples)) => {
                assert!(!samples.is_empty(), "Should have at least one sample");
                assert!(!samples[0].data.is_empty(), "Audio data should not be empty");
                assert!(samples[0].mime_type.starts_with("audio/"), "Should have audio MIME type");
                eprintln!("Generated {} audio sample(s)", samples.len());
            }
            Ok(other) => panic!("Expected Base64 result, got {:?}", other),
            Err(e) => panic!("Music generation failed: {}", e),
        }
    }

    /// Test music generation saving to local file.
    /// Note: This test is expensive and slow, so it's ignored by default.
    #[tokio::test]
    #[ignore = "Expensive API call - run manually with: cargo test --package adk-rust-mcp-music --test integration_test lyria_api_tests::test_music_generation_local_file -- --ignored"]
    async fn test_music_generation_local_file() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let handler = MusicHandler::new(config).await.expect("Failed to create handler");
        
        let output_dir = get_test_output_dir();
        let id = uuid_v4();
        let output_path = output_dir.join(format!("test_music_{}.wav", id));
        
        let params = MusicGenerateParams {
            prompt: "A calm ambient soundscape with soft pads".to_string(),
            negative_prompt: None,
            seed: Some(54321),
            sample_count: 1,
            output_file: Some(output_path.to_string_lossy().to_string()),
            output_gcs_uri: None,
        };
        
        eprintln!("Starting music generation to file (this may take a while)...");
        let result = handler.generate_music(params).await;
        
        match result {
            Ok(MusicGenerateResult::LocalFiles(paths)) => {
                assert_eq!(paths.len(), 1, "Should have 1 output path");
                let path = std::path::PathBuf::from(&paths[0]);
                assert!(path.exists(), "Output file should exist");
                
                let metadata = std::fs::metadata(&path).expect("Should read file metadata");
                assert!(metadata.len() > 1000, "Audio file should have reasonable size: {} bytes", metadata.len());
                
                eprintln!("Music saved to: {} ({} bytes)", path.display(), metadata.len());
            }
            Ok(other) => panic!("Expected LocalFiles result, got {:?}", other),
            Err(e) => panic!("Music generation failed: {}", e),
        }
    }

    /// Test music generation with multiple samples.
    /// Note: This test is expensive and slow, so it's ignored by default.
    #[tokio::test]
    #[ignore = "Expensive API call - run manually"]
    async fn test_music_generation_multiple_samples() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let handler = MusicHandler::new(config).await.expect("Failed to create handler");
        
        let output_dir = get_test_output_dir();
        let id = uuid_v4();
        let output_path = output_dir.join(format!("test_music_multi_{}.wav", id));
        
        let params = MusicGenerateParams {
            prompt: "A jazz piano melody".to_string(),
            negative_prompt: None,
            seed: Some(99999),
            sample_count: 2,
            output_file: Some(output_path.to_string_lossy().to_string()),
            output_gcs_uri: None,
        };
        
        eprintln!("Starting music generation with 2 samples (this may take a while)...");
        let result = handler.generate_music(params).await;
        
        match result {
            Ok(MusicGenerateResult::LocalFiles(paths)) => {
                assert_eq!(paths.len(), 2, "Should have 2 output paths");
                for path_str in &paths {
                    let path = std::path::PathBuf::from(path_str);
                    assert!(path.exists(), "Output file {} should exist", path_str);
                    eprintln!("Music saved to: {}", path.display());
                }
            }
            Ok(other) => panic!("Expected LocalFiles result, got {:?}", other),
            Err(e) => panic!("Music generation failed: {}", e),
        }
    }

    /// Test music generation to GCS.
    /// Note: This test is expensive and slow, so it's ignored by default.
    #[tokio::test]
    #[ignore = "Expensive API call - run manually"]
    async fn test_music_generation_to_gcs() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let gcs_bucket = match &config.gcs_bucket {
            Some(b) => b.clone(),
            None => {
                eprintln!("Skipping GCS test: GCS_BUCKET not set");
                return;
            }
        };
        
        let handler = MusicHandler::new(config).await.expect("Failed to create handler");
        
        let id = uuid_v4();
        let output_uri = format!("gs://{}/integration-test/music_{}.wav", gcs_bucket, id);
        
        let params = MusicGenerateParams {
            prompt: "A relaxing lo-fi beat".to_string(),
            negative_prompt: None,
            seed: Some(77777),
            sample_count: 1,
            output_file: None,
            output_gcs_uri: Some(output_uri.clone()),
        };
        
        eprintln!("Starting music generation to GCS (this may take a while)...");
        let result = handler.generate_music(params).await;
        
        match result {
            Ok(MusicGenerateResult::GcsUris(uris)) => {
                assert_eq!(uris.len(), 1, "Should have 1 output URI");
                assert!(uris[0].starts_with("gs://"), "Should be a GCS URI");
                eprintln!("Music uploaded to GCS: {}", uris[0]);
            }
            Ok(other) => panic!("Expected GcsUris result, got {:?}", other),
            Err(e) => panic!("Music generation to GCS failed: {}", e),
        }
    }
}
