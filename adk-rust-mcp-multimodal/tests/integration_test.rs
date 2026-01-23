//! Integration tests for adk-rust-mcp-multimodal server.
//!
//! Run with: `cargo test --package adk-rust-mcp-multimodal --test integration_test`
//! Skip in CI: `cargo test --package adk-rust-mcp-multimodal --lib`

use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_multimodal::{
    MultimodalHandler, MultimodalImageParams, MultimodalTtsParams,
};
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

// =============================================================================
// Image Generation Tests
// =============================================================================

#[tokio::test]
async fn test_image_generation_base64() {
    skip_if_no_integration!();

    let config = get_test_config().unwrap();
    let handler = MultimodalHandler::new(config)
        .await
        .expect("Failed to create handler");

    let params = MultimodalImageParams {
        prompt: "A simple red circle on a white background".to_string(),
        model: "gemini-2.5-flash-image".to_string(),
        output_file: None,
    };

    let result = handler.generate_image(params).await;

    match result {
        Ok(adk_rust_mcp_multimodal::ImageGenerateResult::Base64(image)) => {
            assert!(!image.data.is_empty(), "Image data should not be empty");
            assert!(
                image.mime_type.starts_with("image/"),
                "MIME type should be an image type"
            );
            println!("Generated image with MIME type: {}", image.mime_type);
        }
        Ok(other) => panic!("Expected Base64 result, got {:?}", other),
        Err(e) => {
            panic!("Image generation failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_image_generation_to_file() {
    skip_if_no_integration!();

    let config = get_test_config().unwrap();
    let handler = MultimodalHandler::new(config)
        .await
        .expect("Failed to create handler");

    let output_dir = get_test_output_dir();
    let output_path = output_dir.join("multimodal_test_image.png");

    let params = MultimodalImageParams {
        prompt: "A simple blue square on a white background".to_string(),
        model: "gemini-2.5-flash-image".to_string(),
        output_file: Some(output_path.to_string_lossy().to_string()),
    };

    let result = handler.generate_image(params).await;

    match result {
        Ok(adk_rust_mcp_multimodal::ImageGenerateResult::LocalFile(path)) => {
            assert!(
                std::path::Path::new(&path).exists(),
                "Output file should exist"
            );
            println!("Image saved to: {}", path);
        }
        Ok(other) => panic!("Expected LocalFile result, got {:?}", other),
        Err(e) => {
            panic!("Image generation failed: {}", e);
        }
    }
}

// =============================================================================
// TTS Tests
// =============================================================================

#[tokio::test]
async fn test_tts_base64() {
    skip_if_no_integration!();

    let config = get_test_config().unwrap();
    let handler = MultimodalHandler::new(config)
        .await
        .expect("Failed to create handler");

    let params = MultimodalTtsParams {
        text: "Hello, this is a test of the Gemini text to speech system.".to_string(),
        voice: Some("Kore".to_string()),
        style: None,
        model: "gemini-2.5-flash-preview-tts".to_string(),
        output_file: None,
    };

    let result = handler.synthesize_speech(params).await;

    match result {
        Ok(adk_rust_mcp_multimodal::TtsResult::Base64(audio)) => {
            assert!(!audio.data.is_empty(), "Audio data should not be empty");
            assert!(
                audio.mime_type.starts_with("audio/"),
                "MIME type should be an audio type"
            );
            println!("Generated audio with MIME type: {}", audio.mime_type);
        }
        Ok(other) => panic!("Expected Base64 result, got {:?}", other),
        Err(e) => {
            panic!("TTS failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_tts_with_style() {
    skip_if_no_integration!();

    let config = get_test_config().unwrap();
    let handler = MultimodalHandler::new(config)
        .await
        .expect("Failed to create handler");

    let params = MultimodalTtsParams {
        text: "I am so happy to see you today!".to_string(),
        voice: Some("Puck".to_string()),
        style: Some("cheerful".to_string()),
        model: "gemini-2.5-flash-preview-tts".to_string(),
        output_file: None,
    };

    let result = handler.synthesize_speech(params).await;

    match result {
        Ok(adk_rust_mcp_multimodal::TtsResult::Base64(audio)) => {
            assert!(!audio.data.is_empty(), "Audio data should not be empty");
            println!("Generated cheerful audio with MIME type: {}", audio.mime_type);
        }
        Ok(other) => panic!("Expected Base64 result, got {:?}", other),
        Err(e) => {
            panic!("TTS with style failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_tts_to_file() {
    skip_if_no_integration!();

    let config = get_test_config().unwrap();
    let handler = MultimodalHandler::new(config)
        .await
        .expect("Failed to create handler");

    let output_dir = get_test_output_dir();
    let output_path = output_dir.join("multimodal_test_audio.wav");

    let params = MultimodalTtsParams {
        text: "This audio is being saved to a file.".to_string(),
        voice: Some("Kore".to_string()),
        style: None,
        model: "gemini-2.5-flash-preview-tts".to_string(),
        output_file: Some(output_path.to_string_lossy().to_string()),
    };

    let result = handler.synthesize_speech(params).await;

    match result {
        Ok(adk_rust_mcp_multimodal::TtsResult::LocalFile(path)) => {
            assert!(
                std::path::Path::new(&path).exists(),
                "Output file should exist"
            );
            println!("Audio saved to: {}", path);
        }
        Ok(other) => panic!("Expected LocalFile result, got {:?}", other),
        Err(e) => {
            panic!("TTS to file failed: {}", e);
        }
    }
}

// =============================================================================
// Voice and Language Code Tests
// =============================================================================

#[tokio::test]
async fn test_list_voices() {
    skip_if_no_integration!();

    let config = get_test_config().unwrap();
    let handler = MultimodalHandler::new(config)
        .await
        .expect("Failed to create handler");

    let voices = handler.list_voices();

    assert!(!voices.is_empty(), "Should have at least one voice");
    println!("Available voices: {:?}", voices);

    // Check that expected voices are present
    let voice_names: Vec<&str> = voices.iter().map(|v| v.name.as_str()).collect();
    assert!(voice_names.contains(&"Kore"), "Should have Kore voice");
    assert!(voice_names.contains(&"Puck"), "Should have Puck voice");
}

#[tokio::test]
async fn test_list_language_codes() {
    skip_if_no_integration!();

    let config = get_test_config().unwrap();
    let handler = MultimodalHandler::new(config)
        .await
        .expect("Failed to create handler");

    let codes = handler.list_language_codes();

    assert!(!codes.is_empty(), "Should have at least one language code");
    println!("Supported language codes: {:?}", codes);

    // Check that expected codes are present
    let code_values: Vec<&str> = codes.iter().map(|c| c.code.as_str()).collect();
    assert!(code_values.contains(&"en-US"), "Should have en-US");
}

// =============================================================================
// Validation Tests (don't require API)
// =============================================================================

#[test]
fn test_image_params_validation_empty_prompt() {
    let params = MultimodalImageParams {
        prompt: "".to_string(),
        model: "test-model".to_string(),
        output_file: None,
    };

    let result = params.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "prompt"));
}

#[test]
fn test_tts_params_validation_empty_text() {
    let params = MultimodalTtsParams {
        text: "".to_string(),
        voice: None,
        style: None,
        model: "test-model".to_string(),
        output_file: None,
    };

    let result = params.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "text"));
}

#[test]
fn test_tts_params_validation_invalid_voice() {
    let params = MultimodalTtsParams {
        text: "Hello".to_string(),
        voice: Some("InvalidVoice".to_string()),
        style: None,
        model: "test-model".to_string(),
        output_file: None,
    };

    let result = params.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "voice"));
}

#[test]
fn test_tts_params_validation_invalid_style() {
    let params = MultimodalTtsParams {
        text: "Hello".to_string(),
        voice: None,
        style: Some("invalid_style".to_string()),
        model: "test-model".to_string(),
        output_file: None,
    };

    let result = params.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "style"));
}

#[test]
fn test_tts_params_validation_valid() {
    let params = MultimodalTtsParams {
        text: "Hello world".to_string(),
        voice: Some("Kore".to_string()),
        style: Some("cheerful".to_string()),
        model: "test-model".to_string(),
        output_file: None,
    };

    assert!(params.validate().is_ok());
}
