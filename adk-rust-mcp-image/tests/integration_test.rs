//! Integration tests for adk-rust-mcp-image server.
//!
//! These tests require:
//! - An authenticated Google Cloud SDK (`gcloud auth application-default login`)
//! - PROJECT_ID environment variable set (or uses gcloud default project)
//! - Vertex AI API enabled in the project
//!
//! Run with: `cargo test --package adk-rust-mcp-image --test integration_test`
//!
//! To skip integration tests in CI, use: `cargo test --package adk-rust-mcp-image --lib`
//!
//! Generated images are saved to `./test_output/` directory for inspection.

use adk_rust_mcp_common::auth::AuthProvider;
use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_common::gcs::GcsClient;
use std::env;
use std::path::PathBuf;
use std::sync::Once;

static INIT: Once = Once::new();

/// Output directory for test-generated images
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
    // Skip if explicitly disabled
    if env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return false;
    }
    
    // Check if we have valid config
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

mod imagen_api_tests {
    use super::*;
    use adk_rust_mcp_image::handler::{ImageGenerateParams, ImageHandler, ImageGenerateResult};

    /// The current Imagen 4 model ID
    const IMAGEN_4_MODEL: &str = "imagen-4.0-generate-preview-06-06";

    /// Helper to save base64 images to test output directory
    fn save_test_images(images: &[adk_rust_mcp_image::GeneratedImage], prefix: &str) {
        let output_dir = get_test_output_dir();
        for (i, img) in images.iter().enumerate() {
            let ext = if img.mime_type.contains("png") { "png" } else { "jpg" };
            let filename = if images.len() == 1 {
                format!("{}.{}", prefix, ext)
            } else {
                format!("{}_{}.{}", prefix, i, ext)
            };
            let path = output_dir.join(&filename);
            
            if let Ok(data) = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                &img.data
            ) {
                if let Err(e) = std::fs::write(&path, &data) {
                    eprintln!("Failed to save {}: {}", filename, e);
                } else {
                    eprintln!("Saved: {}", path.display());
                }
            }
        }
    }

    /// Test basic image generation with Imagen API.
    /// This test actually calls the Vertex AI API.
    #[tokio::test]
    async fn test_generate_image_basic() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let handler = ImageHandler::new(config).await.expect("Failed to create handler");

        let params = ImageGenerateParams {
            prompt: "A simple red circle on a white background".to_string(),
            negative_prompt: None,
            model: IMAGEN_4_MODEL.to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 1,
            seed: None, // Seed not supported with watermark enabled
            output_file: None,
            output_uri: None,
        };

        let result = handler.generate_image(params).await;
        
        match result {
            Ok(ImageGenerateResult::Base64(images)) => {
                assert_eq!(images.len(), 1, "Should generate exactly 1 image");
                assert!(!images[0].data.is_empty(), "Image data should not be empty");
                assert!(images[0].mime_type.starts_with("image/"), "Should have image MIME type");
                
                // Save to test output
                save_test_images(&images, "basic_red_circle");
                
                // Verify it's valid base64
                let decoded = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    &images[0].data
                );
                assert!(decoded.is_ok(), "Should be valid base64 data");
                
                // PNG files start with specific magic bytes
                let bytes = decoded.unwrap();
                assert!(bytes.len() > 8, "Image should have reasonable size");
            }
            Ok(other) => panic!("Expected Base64 result, got {:?}", other),
            Err(e) => panic!("Image generation failed: {}", e),
        }
    }

    /// Test image generation with multiple images.
    #[tokio::test]
    async fn test_generate_multiple_images() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let handler = ImageHandler::new(config).await.expect("Failed to create handler");

        let params = ImageGenerateParams {
            prompt: "A blue square".to_string(),
            negative_prompt: None,
            model: IMAGEN_4_MODEL.to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 2,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = handler.generate_image(params).await;
        
        match result {
            Ok(ImageGenerateResult::Base64(images)) => {
                assert_eq!(images.len(), 2, "Should generate exactly 2 images");
                for (i, img) in images.iter().enumerate() {
                    assert!(!img.data.is_empty(), "Image {} data should not be empty", i);
                }
                
                // Save to test output
                save_test_images(&images, "multiple_blue_square");
            }
            Ok(other) => panic!("Expected Base64 result, got {:?}", other),
            Err(e) => panic!("Image generation failed: {}", e),
        }
    }

    /// Test image generation with different aspect ratios.
    #[tokio::test]
    async fn test_generate_image_aspect_ratios() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let handler = ImageHandler::new(config).await.expect("Failed to create handler");

        // Test 16:9 aspect ratio
        let params = ImageGenerateParams {
            prompt: "A landscape scene with mountains and a sunset".to_string(),
            negative_prompt: None,
            model: IMAGEN_4_MODEL.to_string(),
            aspect_ratio: "16:9".to_string(),
            number_of_images: 1,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = handler.generate_image(params).await;
        
        match result {
            Ok(ImageGenerateResult::Base64(images)) => {
                save_test_images(&images, "landscape_16x9");
            }
            Ok(_) => {}
            Err(e) => panic!("16:9 aspect ratio should work: {}", e),
        }
    }

    /// Test image generation with negative prompt.
    #[tokio::test]
    async fn test_generate_image_with_negative_prompt() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let handler = ImageHandler::new(config).await.expect("Failed to create handler");

        let params = ImageGenerateParams {
            prompt: "A cat sitting on a couch".to_string(),
            negative_prompt: Some("blurry, low quality, distorted".to_string()),
            model: IMAGEN_4_MODEL.to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 1,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = handler.generate_image(params).await;
        
        match result {
            Ok(ImageGenerateResult::Base64(images)) => {
                save_test_images(&images, "cat_on_couch");
            }
            Ok(_) => {}
            Err(e) => panic!("Generation with negative prompt should work: {}", e),
        }
    }

    /// Test saving image to local file.
    #[tokio::test]
    async fn test_generate_image_to_file() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let handler = ImageHandler::new(config).await.expect("Failed to create handler");

        // Use persistent output directory
        let output_dir = get_test_output_dir();
        let output_path = output_dir.join("green_triangle.png");

        let params = ImageGenerateParams {
            prompt: "A green triangle on a black background".to_string(),
            negative_prompt: None,
            model: IMAGEN_4_MODEL.to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 1,
            seed: None,
            output_file: Some(output_path.to_string_lossy().to_string()),
            output_uri: None,
        };

        let result = handler.generate_image(params).await;
        
        match result {
            Ok(ImageGenerateResult::LocalFiles(paths)) => {
                assert_eq!(paths.len(), 1, "Should have 1 output path");
                let path = PathBuf::from(&paths[0]);
                assert!(path.exists(), "Output file should exist");
                
                let metadata = std::fs::metadata(&path).expect("Should read file metadata");
                assert!(metadata.len() > 0, "File should not be empty");
                
                eprintln!("Saved: {}", path.display());
            }
            Ok(other) => panic!("Expected LocalFiles result, got {:?}", other),
            Err(e) => panic!("Image generation failed: {}", e),
        }
    }

    /// Test saving multiple images to local files.
    #[tokio::test]
    async fn test_generate_multiple_images_to_files() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let handler = ImageHandler::new(config).await.expect("Failed to create handler");

        // Use persistent output directory
        let output_dir = get_test_output_dir();
        let output_path = output_dir.join("abstract_art.png");

        let params = ImageGenerateParams {
            prompt: "Abstract colorful art with geometric shapes".to_string(),
            negative_prompt: None,
            model: IMAGEN_4_MODEL.to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 2,
            seed: None,
            output_file: Some(output_path.to_string_lossy().to_string()),
            output_uri: None,
        };

        let result = handler.generate_image(params).await;
        
        match result {
            Ok(ImageGenerateResult::LocalFiles(paths)) => {
                assert_eq!(paths.len(), 2, "Should have 2 output paths");
                for path_str in &paths {
                    let path = PathBuf::from(path_str);
                    assert!(path.exists(), "Output file {} should exist", path_str);
                    eprintln!("Saved: {}", path.display());
                }
            }
            Ok(other) => panic!("Expected LocalFiles result, got {:?}", other),
            Err(e) => panic!("Image generation failed: {}", e),
        }
    }
}

mod auth_tests {
    use super::*;

    /// Test that AuthProvider can get a valid token.
    #[tokio::test]
    async fn test_auth_provider_get_token() {
        skip_if_no_integration!();
        
        let auth = AuthProvider::new().await;
        assert!(auth.is_ok(), "Should create AuthProvider: {:?}", auth.err());
        
        let auth = auth.unwrap();
        let token = auth.get_token(&["https://www.googleapis.com/auth/cloud-platform"]).await;
        
        assert!(token.is_ok(), "Should get token: {:?}", token.err());
        let token = token.unwrap();
        assert!(!token.is_empty(), "Token should not be empty");
    }
}

mod gcs_tests {
    use super::*;
    use adk_rust_mcp_image::handler::{ImageGenerateParams, ImageHandler, ImageGenerateResult};

    /// The current Imagen 4 model ID
    const IMAGEN_4_MODEL: &str = "imagen-4.0-generate-preview-06-06";

    /// Test GCS operations if bucket is configured.
    #[tokio::test]
    async fn test_gcs_upload_download() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let bucket = match &config.gcs_bucket {
            Some(b) => b.clone(),
            None => {
                eprintln!("Skipping GCS test: GCS_BUCKET not set");
                return;
            }
        };

        let auth = AuthProvider::new().await.expect("Failed to create auth");
        let gcs = GcsClient::with_auth(auth);

        // Test data
        let test_data = b"Hello, integration test!";
        let test_path = format!("gs://{}/integration-test/test-{}.txt", bucket, uuid_v4());

        // Parse URI
        let uri = adk_rust_mcp_common::gcs::GcsUri::parse(&test_path)
            .expect("Should parse GCS URI");

        // Upload
        let upload_result = gcs.upload(&uri, test_data, "text/plain").await;
        assert!(upload_result.is_ok(), "Upload should succeed: {:?}", upload_result.err());
        eprintln!("Uploaded to: {}", test_path);

        // Check exists
        let exists = gcs.exists(&uri).await;
        assert!(exists.is_ok(), "Exists check should succeed: {:?}", exists.err());
        assert!(exists.unwrap(), "Object should exist after upload");

        // Download
        let download_result = gcs.download(&uri).await;
        assert!(download_result.is_ok(), "Download should succeed: {:?}", download_result.err());
        assert_eq!(download_result.unwrap(), test_data, "Downloaded data should match");

        eprintln!("GCS upload/download test passed!");
    }

    /// Test generating an image and uploading directly to GCS.
    #[tokio::test]
    async fn test_generate_image_to_gcs() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let bucket = match &config.gcs_bucket {
            Some(b) => b.clone(),
            None => {
                eprintln!("Skipping GCS image test: GCS_BUCKET not set");
                return;
            }
        };

        let handler = ImageHandler::new(config).await.expect("Failed to create handler");

        // Generate unique path for this test
        let timestamp = uuid_v4();
        let output_uri = format!("gs://{}/integration-test/generated-image-{}.png", bucket, timestamp);

        let params = ImageGenerateParams {
            prompt: "A beautiful sunset over the ocean with vibrant orange and purple colors".to_string(),
            negative_prompt: None,
            model: IMAGEN_4_MODEL.to_string(),
            aspect_ratio: "16:9".to_string(),
            number_of_images: 1,
            seed: None,
            output_file: None,
            output_uri: Some(output_uri.clone()),
        };

        let result = handler.generate_image(params).await;
        
        match result {
            Ok(ImageGenerateResult::StorageUris(uris)) => {
                assert_eq!(uris.len(), 1, "Should have 1 output URI");
                eprintln!("Image uploaded to GCS: {}", uris[0]);
                
                // Verify the file exists in GCS
                let auth = AuthProvider::new().await.expect("Failed to create auth");
                let gcs = GcsClient::with_auth(auth);
                let uri = adk_rust_mcp_common::gcs::GcsUri::parse(&uris[0])
                    .expect("Should parse GCS URI");
                
                let exists = gcs.exists(&uri).await;
                assert!(exists.is_ok(), "Exists check should succeed: {:?}", exists.err());
                assert!(exists.unwrap(), "Image should exist in GCS after upload");
                
                // Download and verify it's a valid image
                let data = gcs.download(&uri).await.expect("Should download image");
                assert!(data.len() > 1000, "Image should have reasonable size: {} bytes", data.len());
                
                // Check PNG magic bytes
                assert!(data.starts_with(&[0x89, 0x50, 0x4E, 0x47]), "Should be a valid PNG file");
                
                eprintln!("GCS image generation test passed! Image size: {} bytes", data.len());
            }
            Ok(other) => panic!("Expected StorageUris result, got {:?}", other),
            Err(e) => panic!("Image generation to GCS failed: {}", e),
        }
    }

    /// Test generating multiple images and uploading to GCS.
    #[tokio::test]
    async fn test_generate_multiple_images_to_gcs() {
        skip_if_no_integration!();
        
        let config = get_test_config().unwrap();
        let bucket = match &config.gcs_bucket {
            Some(b) => b.clone(),
            None => {
                eprintln!("Skipping GCS multi-image test: GCS_BUCKET not set");
                return;
            }
        };

        let handler = ImageHandler::new(config).await.expect("Failed to create handler");

        // Generate unique path for this test
        let timestamp = uuid_v4();
        let output_uri = format!("gs://{}/integration-test/multi-image-{}.png", bucket, timestamp);

        let params = ImageGenerateParams {
            prompt: "A futuristic city skyline at night with neon lights".to_string(),
            negative_prompt: None,
            model: IMAGEN_4_MODEL.to_string(),
            aspect_ratio: "16:9".to_string(),
            number_of_images: 2,
            seed: None,
            output_file: None,
            output_uri: Some(output_uri.clone()),
        };

        let result = handler.generate_image(params).await;
        
        match result {
            Ok(ImageGenerateResult::StorageUris(uris)) => {
                assert_eq!(uris.len(), 2, "Should have 2 output URIs");
                
                let auth = AuthProvider::new().await.expect("Failed to create auth");
                let gcs = GcsClient::with_auth(auth);
                
                for (i, uri_str) in uris.iter().enumerate() {
                    eprintln!("Image {} uploaded to GCS: {}", i, uri_str);
                    
                    let uri = adk_rust_mcp_common::gcs::GcsUri::parse(uri_str)
                        .expect("Should parse GCS URI");
                    
                    let exists = gcs.exists(&uri).await;
                    assert!(exists.is_ok() && exists.unwrap(), "Image {} should exist in GCS", i);
                }
                
                eprintln!("GCS multi-image generation test passed!");
            }
            Ok(other) => panic!("Expected StorageUris result, got {:?}", other),
            Err(e) => panic!("Multi-image generation to GCS failed: {}", e),
        }
    }
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
