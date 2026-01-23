//! Integration tests for adk-rust-mcp-speech server.
//!
//! Run with: `cargo test --package adk-rust-mcp-speech --test integration_test`
//! Skip in CI: `cargo test --package adk-rust-mcp-speech --lib`
//!
//! These tests require:
//! - Valid Google Cloud credentials (ADC)
//! - PROJECT_ID environment variable set
//! - Access to Cloud TTS API

use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_speech::handler::{
    Pronunciation, SpeechHandler, SpeechSynthesizeParams, DEFAULT_LANGUAGE_CODE,
    DEFAULT_SPEAKING_RATE, MAX_PITCH, MAX_SPEAKING_RATE, MIN_PITCH, MIN_SPEAKING_RATE,
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

#[tokio::test]
async fn test_handler_creation() {
    skip_if_no_integration!();

    let config = get_test_config().unwrap();
    let handler = SpeechHandler::new(config).await;
    assert!(
        handler.is_ok(),
        "Failed to create handler: {:?}",
        handler.err()
    );
}

#[tokio::test]
async fn test_validation_empty_text() {
    let params = SpeechSynthesizeParams {
        text: "".to_string(),
        voice: None,
        language_code: DEFAULT_LANGUAGE_CODE.to_string(),
        speaking_rate: DEFAULT_SPEAKING_RATE,
        pitch: 0.0,
        pronunciations: None,
        output_file: None,
    };

    let result = params.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "text"));
}

#[tokio::test]
async fn test_validation_invalid_speaking_rate_low() {
    let params = SpeechSynthesizeParams {
        text: "Hello world".to_string(),
        voice: None,
        language_code: DEFAULT_LANGUAGE_CODE.to_string(),
        speaking_rate: 0.1, // Invalid: min is 0.25
        pitch: 0.0,
        pronunciations: None,
        output_file: None,
    };

    let result = params.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "speaking_rate"));
}

#[tokio::test]
async fn test_validation_invalid_speaking_rate_high() {
    let params = SpeechSynthesizeParams {
        text: "Hello world".to_string(),
        voice: None,
        language_code: DEFAULT_LANGUAGE_CODE.to_string(),
        speaking_rate: 5.0, // Invalid: max is 4.0
        pitch: 0.0,
        pronunciations: None,
        output_file: None,
    };

    let result = params.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "speaking_rate"));
}

#[tokio::test]
async fn test_validation_invalid_pitch_low() {
    let params = SpeechSynthesizeParams {
        text: "Hello world".to_string(),
        voice: None,
        language_code: DEFAULT_LANGUAGE_CODE.to_string(),
        speaking_rate: 1.0,
        pitch: -25.0, // Invalid: min is -20.0
        pronunciations: None,
        output_file: None,
    };

    let result = params.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "pitch"));
}

#[tokio::test]
async fn test_validation_invalid_pitch_high() {
    let params = SpeechSynthesizeParams {
        text: "Hello world".to_string(),
        voice: None,
        language_code: DEFAULT_LANGUAGE_CODE.to_string(),
        speaking_rate: 1.0,
        pitch: 25.0, // Invalid: max is 20.0
        pronunciations: None,
        output_file: None,
    };

    let result = params.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "pitch"));
}

#[tokio::test]
async fn test_validation_invalid_pronunciation_alphabet() {
    let params = SpeechSynthesizeParams {
        text: "Hello world".to_string(),
        voice: None,
        language_code: DEFAULT_LANGUAGE_CODE.to_string(),
        speaking_rate: 1.0,
        pitch: 0.0,
        pronunciations: Some(vec![Pronunciation {
            word: "hello".to_string(),
            phonetic: "həˈloʊ".to_string(),
            alphabet: "invalid".to_string(), // Invalid alphabet
        }]),
        output_file: None,
    };

    let result = params.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| e.field.contains("pronunciations") && e.field.contains("alphabet")));
}

#[tokio::test]
async fn test_validation_valid_params() {
    let params = SpeechSynthesizeParams {
        text: "Hello world, this is a test.".to_string(),
        voice: Some("en-US-Chirp3-HD-Achernar".to_string()),
        language_code: "en-US".to_string(),
        speaking_rate: 1.5,
        pitch: 2.0,
        pronunciations: None,
        output_file: None,
    };

    assert!(params.validate().is_ok());
}

#[tokio::test]
async fn test_validation_valid_params_with_pronunciation() {
    let params = SpeechSynthesizeParams {
        text: "I like tomato".to_string(),
        voice: None,
        language_code: "en-US".to_string(),
        speaking_rate: 1.0,
        pitch: 0.0,
        pronunciations: Some(vec![Pronunciation {
            word: "tomato".to_string(),
            phonetic: "təˈmeɪtoʊ".to_string(),
            alphabet: "ipa".to_string(),
        }]),
        output_file: None,
    };

    assert!(params.validate().is_ok());
}

#[tokio::test]
async fn test_validation_boundary_values() {
    // Test minimum valid values
    let params = SpeechSynthesizeParams {
        text: "Test".to_string(),
        voice: None,
        language_code: "en-US".to_string(),
        speaking_rate: MIN_SPEAKING_RATE,
        pitch: MIN_PITCH,
        pronunciations: None,
        output_file: None,
    };
    assert!(params.validate().is_ok());

    // Test maximum valid values
    let params = SpeechSynthesizeParams {
        text: "Test".to_string(),
        voice: None,
        language_code: "en-US".to_string(),
        speaking_rate: MAX_SPEAKING_RATE,
        pitch: MAX_PITCH,
        pronunciations: None,
        output_file: None,
    };
    assert!(params.validate().is_ok());
}

#[tokio::test]
async fn test_ssml_generation() {
    let params = SpeechSynthesizeParams {
        text: "I like tomato".to_string(),
        voice: None,
        language_code: "en-US".to_string(),
        speaking_rate: 1.0,
        pitch: 0.0,
        pronunciations: Some(vec![Pronunciation {
            word: "tomato".to_string(),
            phonetic: "təˈmeɪtoʊ".to_string(),
            alphabet: "ipa".to_string(),
        }]),
        output_file: None,
    };

    let ssml = params.build_ssml();
    assert!(ssml.starts_with("<speak>"));
    assert!(ssml.ends_with("</speak>"));
    assert!(ssml.contains("phoneme"));
    assert!(ssml.contains("ipa"));
    assert!(ssml.contains("təˈmeɪtoʊ"));
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

mod chirp3_api_tests {
    use super::*;
    use adk_rust_mcp_speech::handler::SpeechSynthesizeResult;

    /// Test speech synthesis returning base64 data.
    #[tokio::test]
    #[ignore = "Requires API access - run manually with: cargo test --package adk-rust-mcp-speech --test integration_test chirp3_api_tests::test_speech_synthesis_base64 -- --ignored"]
    async fn test_speech_synthesis_base64() {
        skip_if_no_integration!();

        let config = get_test_config().unwrap();
        let handler = SpeechHandler::new(config)
            .await
            .expect("Failed to create handler");

        let params = SpeechSynthesizeParams {
            text: "Hello, this is a test of the speech synthesis API.".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: 0.0,
            pronunciations: None,
            output_file: None,
        };

        eprintln!("Starting speech synthesis...");
        let result = handler.synthesize(params).await;

        match result {
            Ok(SpeechSynthesizeResult::Base64(audio)) => {
                assert!(!audio.data.is_empty(), "Audio data should not be empty");
                assert!(
                    audio.mime_type.starts_with("audio/"),
                    "Should have audio MIME type"
                );
                eprintln!("Generated audio with MIME type: {}", audio.mime_type);
            }
            Ok(other) => panic!("Expected Base64 result, got {:?}", other),
            Err(e) => panic!("Speech synthesis failed: {}", e),
        }
    }

    /// Test speech synthesis saving to local file.
    #[tokio::test]
    #[ignore = "Requires API access - run manually with: cargo test --package adk-rust-mcp-speech --test integration_test chirp3_api_tests::test_speech_synthesis_local_file -- --ignored"]
    async fn test_speech_synthesis_local_file() {
        skip_if_no_integration!();

        let config = get_test_config().unwrap();
        let handler = SpeechHandler::new(config)
            .await
            .expect("Failed to create handler");

        let output_dir = get_test_output_dir();
        let id = uuid_v4();
        let output_path = output_dir.join(format!("test_speech_{}.wav", id));

        let params = SpeechSynthesizeParams {
            text: "This audio will be saved to a local file for testing purposes.".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: 0.0,
            pronunciations: None,
            output_file: Some(output_path.to_string_lossy().to_string()),
        };

        eprintln!("Starting speech synthesis to file...");
        let result = handler.synthesize(params).await;

        match result {
            Ok(SpeechSynthesizeResult::LocalFile(path)) => {
                let file_path = std::path::PathBuf::from(&path);
                assert!(file_path.exists(), "Output file should exist");

                let metadata = std::fs::metadata(&file_path).expect("Should read file metadata");
                assert!(
                    metadata.len() > 1000,
                    "Audio file should have reasonable size: {} bytes",
                    metadata.len()
                );

                eprintln!("Speech saved to: {} ({} bytes)", path, metadata.len());
            }
            Ok(other) => panic!("Expected LocalFile result, got {:?}", other),
            Err(e) => panic!("Speech synthesis failed: {}", e),
        }
    }

    /// Test speech synthesis with custom speaking rate and pitch.
    #[tokio::test]
    #[ignore = "Requires API access - run manually"]
    async fn test_speech_synthesis_with_rate_and_pitch() {
        skip_if_no_integration!();

        let config = get_test_config().unwrap();
        let handler = SpeechHandler::new(config)
            .await
            .expect("Failed to create handler");

        let output_dir = get_test_output_dir();
        let id = uuid_v4();
        let output_path = output_dir.join(format!("test_speech_rate_pitch_{}.wav", id));

        let params = SpeechSynthesizeParams {
            text: "This is spoken faster and at a higher pitch.".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.5,
            pitch: 5.0,
            pronunciations: None,
            output_file: Some(output_path.to_string_lossy().to_string()),
        };

        let result = handler.synthesize(params).await;

        match result {
            Ok(SpeechSynthesizeResult::LocalFile(path)) => {
                let file_path = std::path::PathBuf::from(&path);
                assert!(file_path.exists(), "Output file should exist");
                eprintln!("Speech with rate/pitch saved to: {}", path);
            }
            Ok(other) => panic!("Expected LocalFile result, got {:?}", other),
            Err(e) => panic!("Speech synthesis failed: {}", e),
        }
    }

    /// Test speech synthesis with custom pronunciation.
    #[tokio::test]
    #[ignore = "Requires API access - run manually"]
    async fn test_speech_synthesis_with_pronunciation() {
        skip_if_no_integration!();

        let config = get_test_config().unwrap();
        let handler = SpeechHandler::new(config)
            .await
            .expect("Failed to create handler");

        let output_dir = get_test_output_dir();
        let id = uuid_v4();
        let output_path = output_dir.join(format!("test_speech_pronunciation_{}.wav", id));

        let params = SpeechSynthesizeParams {
            text: "I like tomato with my pasta.".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: 0.0,
            pronunciations: Some(vec![Pronunciation {
                word: "tomato".to_string(),
                phonetic: "təˈmeɪtoʊ".to_string(),
                alphabet: "ipa".to_string(),
            }]),
            output_file: Some(output_path.to_string_lossy().to_string()),
        };

        let result = handler.synthesize(params).await;

        match result {
            Ok(SpeechSynthesizeResult::LocalFile(path)) => {
                let file_path = std::path::PathBuf::from(&path);
                assert!(file_path.exists(), "Output file should exist");
                eprintln!("Speech with pronunciation saved to: {}", path);
            }
            Ok(other) => panic!("Expected LocalFile result, got {:?}", other),
            Err(e) => panic!("Speech synthesis failed: {}", e),
        }
    }

    /// Test listing available voices.
    #[tokio::test]
    #[ignore = "Requires API access - run manually with: cargo test --package adk-rust-mcp-speech --test integration_test chirp3_api_tests::test_list_voices -- --ignored"]
    async fn test_list_voices() {
        skip_if_no_integration!();

        let config = get_test_config().unwrap();
        let handler = SpeechHandler::new(config)
            .await
            .expect("Failed to create handler");

        let result = handler.list_voices().await;
        assert!(result.is_ok(), "List voices failed: {:?}", result.err());

        let voices = result.unwrap();
        // Should have at least some Chirp3-HD voices
        assert!(!voices.is_empty(), "No Chirp3-HD voices found");
        
        eprintln!("Found {} Chirp3-HD voices:", voices.len());
        for voice in &voices {
            eprintln!("  - {} (languages: {:?})", voice.name, voice.language_codes);
        }
    }
}
