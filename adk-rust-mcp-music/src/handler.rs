//! Music generation handler for the MCP Music server.
//!
//! This module provides the `MusicHandler` struct and parameter types for
//! music generation using Google's Vertex AI Lyria API.

use adk_rust_mcp_common::auth::AuthProvider;
use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_common::error::Error;
use adk_rust_mcp_common::gcs::{GcsClient, GcsUri};
use adk_rust_mcp_common::models::{LyriaModel, ModelRegistry};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, instrument};

/// Default model for music generation.
pub const DEFAULT_MODEL: &str = "lyria-1.0";

/// Minimum number of samples that can be generated.
pub const MIN_SAMPLE_COUNT: u8 = 1;

/// Maximum number of samples that can be generated.
pub const MAX_SAMPLE_COUNT: u8 = 4;

/// Music generation parameters.
///
/// These parameters control the music generation process via the Vertex AI Lyria API.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct MusicGenerateParams {
    /// Text prompt describing the music to generate.
    pub prompt: String,

    /// Negative prompt - what to avoid in the generated music.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,

    /// Random seed for reproducible generation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// Number of samples to generate (1-4).
    #[serde(default = "default_sample_count")]
    pub sample_count: u8,

    /// Output file path for saving the WAV locally.
    /// If not specified and output_gcs_uri is not specified, returns base64-encoded data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_file: Option<String>,

    /// Output GCS URI for saving the WAV to cloud storage.
    /// Format: gs://bucket/path/to/output.wav
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_gcs_uri: Option<String>,
}

fn default_sample_count() -> u8 {
    1
}

/// Validation error details for music generation parameters.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// The field that failed validation.
    pub field: String,
    /// Description of the validation failure.
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl MusicGenerateParams {
    /// Validate the parameters against the model constraints.
    ///
    /// # Returns
    /// - `Ok(())` if all parameters are valid
    /// - `Err(Vec<ValidationError>)` with all validation errors
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate prompt is not empty
        if self.prompt.trim().is_empty() {
            errors.push(ValidationError {
                field: "prompt".to_string(),
                message: "Prompt cannot be empty".to_string(),
            });
        }

        // Validate sample_count range
        if self.sample_count < MIN_SAMPLE_COUNT || self.sample_count > MAX_SAMPLE_COUNT {
            errors.push(ValidationError {
                field: "sample_count".to_string(),
                message: format!(
                    "sample_count must be between {} and {}, got {}",
                    MIN_SAMPLE_COUNT, MAX_SAMPLE_COUNT, self.sample_count
                ),
            });
        }

        // Validate output_gcs_uri format if provided
        if let Some(ref uri) = self.output_gcs_uri {
            if !uri.starts_with("gs://") {
                errors.push(ValidationError {
                    field: "output_gcs_uri".to_string(),
                    message: format!(
                        "output_gcs_uri must be a GCS URI starting with 'gs://', got '{}'",
                        uri
                    ),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the resolved model definition.
    pub fn get_model(&self) -> Option<&'static LyriaModel> {
        ModelRegistry::resolve_lyria(DEFAULT_MODEL)
    }
}

/// Music generation handler.
///
/// Handles music generation requests using the Vertex AI Lyria API.
pub struct MusicHandler {
    /// Application configuration.
    pub config: Config,
    /// GCS client for storage operations.
    pub gcs: GcsClient,
    /// HTTP client for API requests.
    pub http: reqwest::Client,
    /// Authentication provider.
    pub auth: AuthProvider,
}

impl MusicHandler {
    /// Create a new MusicHandler with the given configuration.
    ///
    /// # Errors
    /// Returns an error if GCS client or auth provider initialization fails.
    #[instrument(level = "debug", name = "music_handler_new", skip_all)]
    pub async fn new(config: Config) -> Result<Self, Error> {
        debug!("Initializing MusicHandler");

        let auth = AuthProvider::new().await?;
        let gcs = GcsClient::with_auth(AuthProvider::new().await?);
        let http = reqwest::Client::new();

        Ok(Self {
            config,
            gcs,
            http,
            auth,
        })
    }

    /// Create a new MusicHandler with provided dependencies (for testing).
    #[cfg(test)]
    pub fn with_deps(config: Config, gcs: GcsClient, http: reqwest::Client, auth: AuthProvider) -> Self {
        Self {
            config,
            gcs,
            http,
            auth,
        }
    }

    /// Get the Vertex AI Lyria API endpoint.
    pub fn get_endpoint(&self) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:predict",
            self.config.location,
            self.config.project_id,
            self.config.location,
            "lyria-002"
        )
    }

    /// Generate music from a text prompt.
    ///
    /// # Arguments
    /// * `params` - Music generation parameters
    ///
    /// # Returns
    /// * `Ok(MusicGenerateResult)` - Generated music with their data or paths
    /// * `Err(Error)` - If validation fails, API call fails, or output handling fails
    #[instrument(level = "info", name = "generate_music", skip(self, params))]
    pub async fn generate_music(&self, params: MusicGenerateParams) -> Result<MusicGenerateResult, Error> {
        // Validate parameters
        params.validate().map_err(|errors| {
            let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            Error::validation(messages.join("; "))
        })?;

        info!(sample_count = params.sample_count, "Generating music with Lyria API");

        // Build the API request
        let request = LyriaRequest {
            instances: vec![LyriaInstance {
                prompt: params.prompt.clone(),
                negative_prompt: params.negative_prompt.clone(),
            }],
            parameters: LyriaParameters {
                sample_count: params.sample_count,
                seed: params.seed,
            },
        };

        // Get auth token
        let token = self.auth.get_token(&["https://www.googleapis.com/auth/cloud-platform"]).await?;

        // Make API request
        let endpoint = self.get_endpoint();
        debug!(endpoint = %endpoint, "Calling Lyria API");

        let response = self.http
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::api(&endpoint, 0, format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::api(&endpoint, status.as_u16(), body));
        }

        // Get raw response for debugging
        let response_text = response.text().await.map_err(|e| {
            Error::api(&endpoint, status.as_u16(), format!("Failed to read response: {}", e))
        })?;
        
        debug!(response = %response_text.chars().take(500).collect::<String>(), "Raw Lyria API response");

        // Parse response
        let api_response: LyriaResponse = serde_json::from_str(&response_text).map_err(|e| {
            Error::api(&endpoint, status.as_u16(), format!("Failed to parse response: {}. Raw: {}", e, &response_text[..response_text.len().min(500)]))
        })?;

        // Extract audio samples from response
        let samples: Vec<GeneratedAudio> = api_response
            .predictions
            .into_iter()
            .filter_map(|p| {
                p.bytes_base64_encoded.map(|data| GeneratedAudio {
                    data,
                    mime_type: p.mime_type.unwrap_or_else(|| "audio/wav".to_string()),
                })
            })
            .collect();

        if samples.is_empty() {
            return Err(Error::api(&endpoint, 200, "No audio samples returned from API"));
        }

        info!(count = samples.len(), "Received audio samples from API");

        // Handle output based on params
        self.handle_output(samples, &params).await
    }

    /// Handle output of generated audio samples based on params.
    async fn handle_output(
        &self,
        samples: Vec<GeneratedAudio>,
        params: &MusicGenerateParams,
    ) -> Result<MusicGenerateResult, Error> {
        // If output_gcs_uri is specified, upload to GCS
        if let Some(output_uri) = &params.output_gcs_uri {
            return self.upload_to_gcs(samples, output_uri).await;
        }

        // If output_file is specified, save to local file
        if let Some(output_file) = &params.output_file {
            return self.save_to_file(samples, output_file).await;
        }

        // Otherwise, return base64-encoded data
        Ok(MusicGenerateResult::Base64(samples))
    }

    /// Upload audio samples to GCS.
    async fn upload_to_gcs(
        &self,
        samples: Vec<GeneratedAudio>,
        output_uri: &str,
    ) -> Result<MusicGenerateResult, Error> {
        let mut uris = Vec::new();

        for (i, sample) in samples.iter().enumerate() {
            // Decode base64 data
            let data = BASE64.decode(&sample.data).map_err(|e| {
                Error::validation(format!("Invalid base64 data: {}", e))
            })?;

            // Determine the URI for this sample
            let uri = if samples.len() == 1 {
                output_uri.to_string()
            } else {
                // Add index suffix for multiple samples
                // Handle GCS URIs properly - don't use Path which treats gs:// as filesystem path
                Self::add_index_suffix_to_gcs_uri(output_uri, i, "audio", "wav")
            };

            // Parse GCS URI and upload
            let gcs_uri = GcsUri::parse(&uri)?;
            self.gcs.upload(&gcs_uri, &data, &sample.mime_type).await?;
            uris.push(uri);
        }

        info!(count = uris.len(), "Uploaded audio samples to GCS");
        Ok(MusicGenerateResult::GcsUris(uris))
    }

    /// Add an index suffix to a GCS URI for multi-output scenarios.
    fn add_index_suffix_to_gcs_uri(uri: &str, index: usize, default_stem: &str, default_ext: &str) -> String {
        // For GCS URIs, extract the path portion after gs://bucket/
        if let Some(stripped) = uri.strip_prefix("gs://") {
            if let Some(slash_pos) = stripped.find('/') {
                let bucket = &stripped[..slash_pos];
                let object_path = &stripped[slash_pos + 1..];
                
                // Find the last component (filename)
                let (dir, filename) = if let Some(last_slash) = object_path.rfind('/') {
                    (&object_path[..last_slash], &object_path[last_slash + 1..])
                } else {
                    ("", object_path)
                };
                
                // Split filename into stem and extension
                let (stem, ext) = if let Some(dot_pos) = filename.rfind('.') {
                    (&filename[..dot_pos], &filename[dot_pos + 1..])
                } else {
                    (filename, default_ext)
                };
                
                let stem = if stem.is_empty() { default_stem } else { stem };
                
                if dir.is_empty() {
                    format!("gs://{}/{}_{}.{}", bucket, stem, index, ext)
                } else {
                    format!("gs://{}/{}/{}_{}.{}", bucket, dir, stem, index, ext)
                }
            } else {
                // Malformed GCS URI (no path after bucket), just append index
                format!("{}/{}_{}.{}", uri, default_stem, index, default_ext)
            }
        } else {
            // Shouldn't happen since we validate GCS URIs, but handle gracefully
            format!("{}_{}", uri, index)
        }
    }

    /// Save audio samples to local files.
    async fn save_to_file(
        &self,
        samples: Vec<GeneratedAudio>,
        output_file: &str,
    ) -> Result<MusicGenerateResult, Error> {
        let mut paths = Vec::new();

        for (i, sample) in samples.iter().enumerate() {
            // Decode base64 data
            let data = BASE64.decode(&sample.data).map_err(|e| {
                Error::validation(format!("Invalid base64 data: {}", e))
            })?;

            // Determine the path for this sample
            let path = if samples.len() == 1 {
                output_file.to_string()
            } else {
                // Add index suffix for multiple samples
                let p = Path::new(output_file);
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("audio");
                let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("wav");
                let parent = p.parent().and_then(|p| p.to_str()).unwrap_or("");
                if parent.is_empty() {
                    format!("{}_{}.{}", stem, i, ext)
                } else {
                    format!("{}/{}_{}.{}", parent, stem, i, ext)
                }
            };

            // Ensure parent directory exists
            if let Some(parent) = Path::new(&path).parent() {
                if !parent.as_os_str().is_empty() {
                    tokio::fs::create_dir_all(parent).await?;
                }
            }

            // Write to file
            tokio::fs::write(&path, &data).await?;
            paths.push(path);
        }

        info!(count = paths.len(), "Saved audio samples to local files");
        Ok(MusicGenerateResult::LocalFiles(paths))
    }
}

// =============================================================================
// API Request/Response Types
// =============================================================================

/// Vertex AI Lyria API request.
#[derive(Debug, Serialize)]
pub struct LyriaRequest {
    /// Input instances (prompts)
    pub instances: Vec<LyriaInstance>,
    /// Generation parameters
    pub parameters: LyriaParameters,
}

/// Lyria API instance (prompt).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyriaInstance {
    /// Text prompt describing the music
    pub prompt: String,
    /// Negative prompt - what to avoid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
}

/// Lyria API parameters.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyriaParameters {
    /// Number of samples to generate
    pub sample_count: u8,
    /// Random seed for reproducibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

/// Vertex AI Lyria API response.
#[derive(Debug, Deserialize)]
pub struct LyriaResponse {
    /// Generated audio predictions
    pub predictions: Vec<LyriaPrediction>,
}

/// Lyria API prediction (generated audio).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyriaPrediction {
    /// Base64-encoded audio data
    pub bytes_base64_encoded: Option<String>,
    /// MIME type of the audio
    pub mime_type: Option<String>,
}

// =============================================================================
// Result Types
// =============================================================================

/// Generated audio data.
#[derive(Debug, Clone)]
pub struct GeneratedAudio {
    /// Base64-encoded audio data
    pub data: String,
    /// MIME type of the audio
    pub mime_type: String,
}

/// Result of music generation.
#[derive(Debug)]
pub enum MusicGenerateResult {
    /// Base64-encoded audio data (when no output specified)
    Base64(Vec<GeneratedAudio>),
    /// Local file paths (when output_file specified)
    LocalFiles(Vec<String>),
    /// GCS URIs (when output_gcs_uri specified)
    GcsUris(Vec<String>),
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params: MusicGenerateParams = serde_json::from_str(r#"{"prompt": "upbeat jazz"}"#).unwrap();
        assert_eq!(params.sample_count, 1);
        assert!(params.negative_prompt.is_none());
        assert!(params.seed.is_none());
        assert!(params.output_file.is_none());
        assert!(params.output_gcs_uri.is_none());
    }

    #[test]
    fn test_valid_params() {
        let params = MusicGenerateParams {
            prompt: "A relaxing piano melody".to_string(),
            negative_prompt: Some("drums, loud".to_string()),
            seed: Some(42),
            sample_count: 2,
            output_file: None,
            output_gcs_uri: None,
        };

        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_invalid_sample_count_zero() {
        let params = MusicGenerateParams {
            prompt: "A song".to_string(),
            negative_prompt: None,
            seed: None,
            sample_count: 0,
            output_file: None,
            output_gcs_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "sample_count"));
    }

    #[test]
    fn test_invalid_sample_count_too_high() {
        let params = MusicGenerateParams {
            prompt: "A song".to_string(),
            negative_prompt: None,
            seed: None,
            sample_count: 5,
            output_file: None,
            output_gcs_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "sample_count"));
    }

    #[test]
    fn test_empty_prompt() {
        let params = MusicGenerateParams {
            prompt: "   ".to_string(),
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

    #[test]
    fn test_invalid_gcs_uri() {
        let params = MusicGenerateParams {
            prompt: "A song".to_string(),
            negative_prompt: None,
            seed: None,
            sample_count: 1,
            output_file: None,
            output_gcs_uri: Some("/local/path/output.wav".to_string()),
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "output_gcs_uri"));
    }

    #[test]
    fn test_valid_gcs_uri() {
        let params = MusicGenerateParams {
            prompt: "A song".to_string(),
            negative_prompt: None,
            seed: None,
            sample_count: 1,
            output_file: None,
            output_gcs_uri: Some("gs://bucket/output.wav".to_string()),
        };

        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_all_valid_sample_counts() {
        for n in MIN_SAMPLE_COUNT..=MAX_SAMPLE_COUNT {
            let params = MusicGenerateParams {
                prompt: "A song".to_string(),
                negative_prompt: None,
                seed: None,
                sample_count: n,
                output_file: None,
                output_gcs_uri: None,
            };
            assert!(params.validate().is_ok(), "sample_count {} should be valid", n);
        }
    }

    #[test]
    fn test_serialization_roundtrip() {
        let params = MusicGenerateParams {
            prompt: "A jazz tune".to_string(),
            negative_prompt: Some("vocals".to_string()),
            seed: Some(42),
            sample_count: 2,
            output_file: Some("/tmp/output.wav".to_string()),
            output_gcs_uri: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: MusicGenerateParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params.prompt, deserialized.prompt);
        assert_eq!(params.negative_prompt, deserialized.negative_prompt);
        assert_eq!(params.seed, deserialized.seed);
        assert_eq!(params.sample_count, deserialized.sample_count);
        assert_eq!(params.output_file, deserialized.output_file);
    }

    // Tests for GCS URI handling (P1 fix)
    #[test]
    fn test_add_index_suffix_to_gcs_uri_simple() {
        let uri = "gs://bucket/output.wav";
        let result = MusicHandler::add_index_suffix_to_gcs_uri(uri, 0, "audio", "wav");
        assert_eq!(result, "gs://bucket/output_0.wav");
    }

    #[test]
    fn test_add_index_suffix_to_gcs_uri_with_path() {
        let uri = "gs://bucket/path/to/output.wav";
        let result = MusicHandler::add_index_suffix_to_gcs_uri(uri, 1, "audio", "wav");
        assert_eq!(result, "gs://bucket/path/to/output_1.wav");
    }

    #[test]
    fn test_add_index_suffix_to_gcs_uri_no_extension() {
        let uri = "gs://bucket/output";
        let result = MusicHandler::add_index_suffix_to_gcs_uri(uri, 2, "audio", "wav");
        assert_eq!(result, "gs://bucket/output_2.wav");
    }

    #[test]
    fn test_add_index_suffix_preserves_gs_prefix() {
        // This is the key test for the P1 bug - ensure gs:// is preserved, not mangled to gs:/
        let uri = "gs://my-bucket/folder/music.wav";
        let result = MusicHandler::add_index_suffix_to_gcs_uri(uri, 0, "audio", "wav");
        assert!(result.starts_with("gs://"), "URI should start with gs://, got: {}", result);
        assert_eq!(result, "gs://my-bucket/folder/music_0.wav");
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: rust-mcp-genmedia, Property 8: Numeric Parameter Range Validation (sample_count)
    // **Validates: Requirements 6.5**
    //
    // For any numeric parameter with defined bounds (sample_count 1-4),
    // values outside the valid range SHALL be rejected with a validation error.

    /// Strategy to generate valid sample_count values (1-4)
    fn valid_sample_count_strategy() -> impl Strategy<Value = u8> {
        MIN_SAMPLE_COUNT..=MAX_SAMPLE_COUNT
    }

    /// Strategy to generate invalid sample_count values (0 or > 4)
    fn invalid_sample_count_strategy() -> impl Strategy<Value = u8> {
        prop_oneof![
            Just(0u8),
            (MAX_SAMPLE_COUNT + 1)..=u8::MAX,
        ]
    }

    /// Strategy to generate valid prompts (non-empty)
    fn valid_prompt_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 ]{1,100}".prop_map(|s| s.trim().to_string())
            .prop_filter("Must not be empty", |s| !s.trim().is_empty())
    }

    proptest! {
        /// Property 8: Valid sample_count values (1-4) should pass validation
        #[test]
        fn valid_sample_count_passes_validation(
            num in valid_sample_count_strategy(),
            prompt in valid_prompt_strategy(),
        ) {
            let params = MusicGenerateParams {
                prompt,
                negative_prompt: None,
                seed: None,
                sample_count: num,
                output_file: None,
                output_gcs_uri: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_ok(),
                "sample_count {} should be valid, but got errors: {:?}",
                num,
                result.err()
            );
        }

        /// Property 8: Invalid sample_count values (0 or > 4) should fail validation
        #[test]
        fn invalid_sample_count_fails_validation(
            num in invalid_sample_count_strategy(),
            prompt in valid_prompt_strategy(),
        ) {
            let params = MusicGenerateParams {
                prompt,
                negative_prompt: None,
                seed: None,
                sample_count: num,
                output_file: None,
                output_gcs_uri: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_err(),
                "sample_count {} should be invalid",
                num
            );

            let errors = result.unwrap_err();
            prop_assert!(
                errors.iter().any(|e| e.field == "sample_count"),
                "Should have a sample_count validation error for value {}",
                num
            );
        }

        /// Property: Empty prompts should always fail validation regardless of sample_count
        #[test]
        fn empty_prompt_fails_validation(
            num in valid_sample_count_strategy(),
        ) {
            let params = MusicGenerateParams {
                prompt: "   ".to_string(),
                negative_prompt: None,
                seed: None,
                sample_count: num,
                output_file: None,
                output_gcs_uri: None,
            };

            let result = params.validate();
            prop_assert!(result.is_err());

            let errors = result.unwrap_err();
            prop_assert!(
                errors.iter().any(|e| e.field == "prompt"),
                "Should have a prompt validation error"
            );
        }

        /// Property: Valid GCS URIs should pass validation
        #[test]
        fn valid_gcs_uri_passes_validation(
            prompt in valid_prompt_strategy(),
            bucket in "[a-z][a-z0-9-]{2,20}",
            path in "[a-z0-9/]{1,30}\\.wav",
        ) {
            let gcs_uri = format!("gs://{}/{}", bucket, path);
            let params = MusicGenerateParams {
                prompt,
                negative_prompt: None,
                seed: None,
                sample_count: 1,
                output_file: None,
                output_gcs_uri: Some(gcs_uri.clone()),
            };

            let result = params.validate();
            prop_assert!(
                result.is_ok(),
                "GCS URI '{}' should be valid, but got errors: {:?}",
                gcs_uri,
                result.err()
            );
        }

        /// Property: Non-GCS URIs should fail validation
        #[test]
        fn invalid_gcs_uri_fails_validation(
            prompt in valid_prompt_strategy(),
            path in "/[a-z0-9/]{1,30}\\.wav",
        ) {
            let params = MusicGenerateParams {
                prompt,
                negative_prompt: None,
                seed: None,
                sample_count: 1,
                output_file: None,
                output_gcs_uri: Some(path.clone()),
            };

            let result = params.validate();
            prop_assert!(
                result.is_err(),
                "Path '{}' should be invalid as GCS URI",
                path
            );

            let errors = result.unwrap_err();
            prop_assert!(
                errors.iter().any(|e| e.field == "output_gcs_uri"),
                "Should have an output_gcs_uri validation error for path '{}'",
                path
            );
        }
    }
}
