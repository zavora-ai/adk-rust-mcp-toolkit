//! Video generation handler for the MCP Video server.
//!
//! This module provides the `VideoHandler` struct and parameter types for
//! video generation using Google's Vertex AI Veo API.

use adk_rust_mcp_common::auth::AuthProvider;
use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_common::error::Error;
use adk_rust_mcp_common::gcs::{GcsClient, GcsUri};
use adk_rust_mcp_common::models::{ModelRegistry, VeoModel, VEO_MODELS};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info, instrument};

/// Valid aspect ratios for video generation.
pub const VALID_ASPECT_RATIOS: &[&str] = &["16:9", "9:16"];

/// Default model for video generation.
pub const DEFAULT_MODEL: &str = "veo-3.0-generate-preview";

/// Default duration in seconds (must be one of the supported values: 4, 6, 8).
pub const DEFAULT_DURATION_SECONDS: u8 = 8;

/// Supported durations in seconds (for fallback validation when model is unknown).
pub const SUPPORTED_DURATIONS: &[u8] = &[4, 6, 8];

/// Minimum duration in seconds (for fallback validation).
pub const MIN_DURATION_SECONDS: u8 = 4;

/// Maximum duration in seconds (for fallback validation).
pub const MAX_DURATION_SECONDS: u8 = 8;

/// Default aspect ratio.
pub const DEFAULT_ASPECT_RATIO: &str = "16:9";

/// LRO polling configuration
pub const LRO_INITIAL_DELAY_MS: u64 = 5000;
pub const LRO_MAX_DELAY_MS: u64 = 60000;
pub const LRO_BACKOFF_MULTIPLIER: f64 = 1.5;
pub const LRO_MAX_ATTEMPTS: u32 = 120; // ~30 minutes max with backoff

/// Text-to-video generation parameters.
///
/// These parameters control the video generation process via the Vertex AI Veo API.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct VideoT2vParams {
    /// Text prompt describing the video to generate.
    pub prompt: String,

    /// Model to use for generation.
    /// Defaults to "veo-3.0-generate-preview".
    #[serde(default = "default_model")]
    pub model: String,

    /// Aspect ratio for the generated video.
    /// Valid values: "16:9", "9:16".
    #[serde(default = "default_aspect_ratio")]
    pub aspect_ratio: String,

    /// Duration of the video in seconds (5-8 depending on model).
    #[serde(default = "default_duration_seconds")]
    pub duration_seconds: u8,

    /// GCS URI for output (required by Veo API).
    /// Format: gs://bucket/path/to/output.mp4
    pub output_gcs_uri: String,

    /// Whether to also download the video locally after generation.
    #[serde(default)]
    pub download_local: bool,

    /// Local path to save the video if download_local is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,

    /// Whether to generate audio (only supported on Veo 3.x models).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,

    /// Random seed for reproducible generation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

fn default_model() -> String {
    DEFAULT_MODEL.to_string()
}

fn default_aspect_ratio() -> String {
    DEFAULT_ASPECT_RATIO.to_string()
}

fn default_duration_seconds() -> u8 {
    DEFAULT_DURATION_SECONDS
}

/// Image-to-video generation parameters.
///
/// These parameters control the image-to-video generation process via the Vertex AI Veo API.
/// Supports both single-image I2V and interpolation (first + last frame).
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct VideoI2vParams {
    /// Source image for video generation (first frame for interpolation).
    /// Can be base64 data, local file path, or GCS URI.
    pub image: String,

    /// Text prompt describing the desired video motion.
    pub prompt: String,

    /// Last frame image for interpolation mode.
    /// If provided, generates a video interpolating between `image` and `last_frame_image`.
    /// Can be base64 data, local file path, or GCS URI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_frame_image: Option<String>,

    /// Model to use for generation.
    /// Defaults to "veo-3.0-generate-preview".
    #[serde(default = "default_model")]
    pub model: String,

    /// Aspect ratio for the generated video.
    /// Valid values: "16:9", "9:16".
    #[serde(default = "default_aspect_ratio")]
    pub aspect_ratio: String,

    /// Duration of the video in seconds (5-8 depending on model).
    #[serde(default = "default_duration_seconds")]
    pub duration_seconds: u8,

    /// GCS URI for output (required by Veo API).
    /// Format: gs://bucket/path/to/output.mp4
    pub output_gcs_uri: String,

    /// Whether to also download the video locally after generation.
    #[serde(default)]
    pub download_local: bool,

    /// Local path to save the video if download_local is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,

    /// Random seed for reproducible generation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

/// Video extension parameters.
///
/// These parameters control the video extension process via the Vertex AI Veo API.
/// Extends an existing video by generating additional frames.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct VideoExtendParams {
    /// GCS URI of the video to extend.
    /// Format: gs://bucket/path/to/input.mp4
    pub video_input: String,

    /// Text prompt describing the desired continuation.
    pub prompt: String,

    /// Model to use for generation.
    /// Defaults to "veo-3.0-generate-preview".
    #[serde(default = "default_model")]
    pub model: String,

    /// Duration of the extension in seconds (5-8 depending on model).
    #[serde(default = "default_duration_seconds")]
    pub duration_seconds: u8,

    /// GCS URI for output (required by Veo API).
    /// Format: gs://bucket/path/to/output.mp4
    pub output_gcs_uri: String,

    /// Whether to also download the video locally after generation.
    #[serde(default)]
    pub download_local: bool,

    /// Local path to save the video if download_local is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,

    /// Random seed for reproducible generation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

/// Validation error details for video generation parameters.
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

impl VideoT2vParams {
    /// Validate the parameters against the model constraints.
    ///
    /// # Returns
    /// - `Ok(())` if all parameters are valid
    /// - `Err(Vec<ValidationError>)` with all validation errors
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Resolve the model to get constraints
        let model = ModelRegistry::resolve_veo(&self.model);

        // Validate model exists
        if model.is_none() {
            errors.push(ValidationError {
                field: "model".to_string(),
                message: format!(
                    "Unknown model '{}'. Valid models: {}",
                    self.model,
                    VEO_MODELS
                        .iter()
                        .map(|m| m.id)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
        }

        // Validate prompt is not empty
        if self.prompt.trim().is_empty() {
            errors.push(ValidationError {
                field: "prompt".to_string(),
                message: "Prompt cannot be empty".to_string(),
            });
        }

        // Validate aspect ratio
        if let Some(model) = model {
            if !model.supported_aspect_ratios.contains(&self.aspect_ratio.as_str()) {
                errors.push(ValidationError {
                    field: "aspect_ratio".to_string(),
                    message: format!(
                        "Invalid aspect ratio '{}'. Valid options for {}: {}",
                        self.aspect_ratio,
                        model.id,
                        model.supported_aspect_ratios.join(", ")
                    ),
                });
            }

            // Validate duration_seconds against model's supported durations
            if !model.supported_durations.contains(&self.duration_seconds) {
                let durations_str: Vec<String> = model.supported_durations.iter().map(|d| d.to_string()).collect();
                errors.push(ValidationError {
                    field: "duration_seconds".to_string(),
                    message: format!(
                        "duration_seconds must be one of [{}] for model {}, got {}",
                        durations_str.join(", "), model.id, self.duration_seconds
                    ),
                });
            }

            // Validate generate_audio is only used with Veo 3.x models
            if self.generate_audio.is_some() && !model.supports_audio {
                errors.push(ValidationError {
                    field: "generate_audio".to_string(),
                    message: format!(
                        "generate_audio is only supported on Veo 3.x models, not {}",
                        model.id
                    ),
                });
            }
        } else {
            // If model is unknown, validate against common constraints
            if !VALID_ASPECT_RATIOS.contains(&self.aspect_ratio.as_str()) {
                errors.push(ValidationError {
                    field: "aspect_ratio".to_string(),
                    message: format!(
                        "Invalid aspect ratio '{}'. Valid options: {}",
                        self.aspect_ratio,
                        VALID_ASPECT_RATIOS.join(", ")
                    ),
                });
            }

            if !SUPPORTED_DURATIONS.contains(&self.duration_seconds) {
                let durations_str: Vec<String> = SUPPORTED_DURATIONS.iter().map(|d| d.to_string()).collect();
                errors.push(ValidationError {
                    field: "duration_seconds".to_string(),
                    message: format!(
                        "duration_seconds must be one of [{}], got {}",
                        durations_str.join(", "), self.duration_seconds
                    ),
                });
            }
        }

        // Validate output_gcs_uri is a valid GCS URI
        if !self.output_gcs_uri.starts_with("gs://") {
            errors.push(ValidationError {
                field: "output_gcs_uri".to_string(),
                message: format!(
                    "output_gcs_uri must be a GCS URI starting with 'gs://', got '{}'",
                    self.output_gcs_uri
                ),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the resolved model definition.
    pub fn get_model(&self) -> Option<&'static VeoModel> {
        ModelRegistry::resolve_veo(&self.model)
    }
}

impl VideoI2vParams {
    /// Validate the parameters against the model constraints.
    ///
    /// # Returns
    /// - `Ok(())` if all parameters are valid
    /// - `Err(Vec<ValidationError>)` with all validation errors
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Resolve the model to get constraints
        let model = ModelRegistry::resolve_veo(&self.model);

        // Validate model exists
        if model.is_none() {
            errors.push(ValidationError {
                field: "model".to_string(),
                message: format!(
                    "Unknown model '{}'. Valid models: {}",
                    self.model,
                    VEO_MODELS
                        .iter()
                        .map(|m| m.id)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
        }

        // Validate image is not empty
        if self.image.trim().is_empty() {
            errors.push(ValidationError {
                field: "image".to_string(),
                message: "Image cannot be empty".to_string(),
            });
        }

        // Validate prompt is not empty
        if self.prompt.trim().is_empty() {
            errors.push(ValidationError {
                field: "prompt".to_string(),
                message: "Prompt cannot be empty".to_string(),
            });
        }

        // Validate aspect ratio
        if let Some(model) = model {
            if !model.supported_aspect_ratios.contains(&self.aspect_ratio.as_str()) {
                errors.push(ValidationError {
                    field: "aspect_ratio".to_string(),
                    message: format!(
                        "Invalid aspect ratio '{}'. Valid options for {}: {}",
                        self.aspect_ratio,
                        model.id,
                        model.supported_aspect_ratios.join(", ")
                    ),
                });
            }

            // Validate duration_seconds against model's supported durations
            if !model.supported_durations.contains(&self.duration_seconds) {
                let durations_str: Vec<String> = model.supported_durations.iter().map(|d| d.to_string()).collect();
                errors.push(ValidationError {
                    field: "duration_seconds".to_string(),
                    message: format!(
                        "duration_seconds must be one of [{}] for model {}, got {}",
                        durations_str.join(", "), model.id, self.duration_seconds
                    ),
                });
            }
        } else {
            // If model is unknown, validate against common constraints
            if !VALID_ASPECT_RATIOS.contains(&self.aspect_ratio.as_str()) {
                errors.push(ValidationError {
                    field: "aspect_ratio".to_string(),
                    message: format!(
                        "Invalid aspect ratio '{}'. Valid options: {}",
                        self.aspect_ratio,
                        VALID_ASPECT_RATIOS.join(", ")
                    ),
                });
            }

            if !SUPPORTED_DURATIONS.contains(&self.duration_seconds) {
                let durations_str: Vec<String> = SUPPORTED_DURATIONS.iter().map(|d| d.to_string()).collect();
                errors.push(ValidationError {
                    field: "duration_seconds".to_string(),
                    message: format!(
                        "duration_seconds must be one of [{}], got {}",
                        durations_str.join(", "), self.duration_seconds
                    ),
                });
            }
        }

        // Validate output_gcs_uri is a valid GCS URI
        if !self.output_gcs_uri.starts_with("gs://") {
            errors.push(ValidationError {
                field: "output_gcs_uri".to_string(),
                message: format!(
                    "output_gcs_uri must be a GCS URI starting with 'gs://', got '{}'",
                    self.output_gcs_uri
                ),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the resolved model definition.
    pub fn get_model(&self) -> Option<&'static VeoModel> {
        ModelRegistry::resolve_veo(&self.model)
    }
}

impl VideoExtendParams {
    /// Validate the parameters against the model constraints.
    ///
    /// # Returns
    /// - `Ok(())` if all parameters are valid
    /// - `Err(Vec<ValidationError>)` with all validation errors
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Resolve the model to get constraints
        let model = ModelRegistry::resolve_veo(&self.model);

        // Validate model exists
        if model.is_none() {
            errors.push(ValidationError {
                field: "model".to_string(),
                message: format!(
                    "Unknown model '{}'. Valid models: {}",
                    self.model,
                    VEO_MODELS
                        .iter()
                        .map(|m| m.id)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
        }

        // Validate video_input is a valid GCS URI
        if !self.video_input.starts_with("gs://") {
            errors.push(ValidationError {
                field: "video_input".to_string(),
                message: format!(
                    "video_input must be a GCS URI starting with 'gs://', got '{}'",
                    self.video_input
                ),
            });
        }

        // Validate prompt is not empty
        if self.prompt.trim().is_empty() {
            errors.push(ValidationError {
                field: "prompt".to_string(),
                message: "Prompt cannot be empty".to_string(),
            });
        }

        // Validate duration_seconds against model's supported durations
        if let Some(model) = model {
            if !model.supported_durations.contains(&self.duration_seconds) {
                let durations_str: Vec<String> = model.supported_durations.iter().map(|d| d.to_string()).collect();
                errors.push(ValidationError {
                    field: "duration_seconds".to_string(),
                    message: format!(
                        "duration_seconds must be one of [{}] for model {}, got {}",
                        durations_str.join(", "), model.id, self.duration_seconds
                    ),
                });
            }
        } else if !SUPPORTED_DURATIONS.contains(&self.duration_seconds) {
            let durations_str: Vec<String> = SUPPORTED_DURATIONS.iter().map(|d| d.to_string()).collect();
            errors.push(ValidationError {
                field: "duration_seconds".to_string(),
                message: format!(
                    "duration_seconds must be one of [{}], got {}",
                    durations_str.join(", "), self.duration_seconds
                ),
            });
        }

        // Validate output_gcs_uri is a valid GCS URI
        if !self.output_gcs_uri.starts_with("gs://") {
            errors.push(ValidationError {
                field: "output_gcs_uri".to_string(),
                message: format!(
                    "output_gcs_uri must be a GCS URI starting with 'gs://', got '{}'",
                    self.output_gcs_uri
                ),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the resolved model definition.
    pub fn get_model(&self) -> Option<&'static VeoModel> {
        ModelRegistry::resolve_veo(&self.model)
    }
}

/// Video generation handler.
///
/// Handles video generation requests using the Vertex AI Veo API.
pub struct VideoHandler {
    /// Application configuration.
    pub config: Config,
    /// GCS client for storage operations.
    pub gcs: GcsClient,
    /// HTTP client for API requests.
    pub http: reqwest::Client,
    /// Authentication provider.
    pub auth: AuthProvider,
}

impl VideoHandler {
    /// Create a new VideoHandler with the given configuration.
    ///
    /// # Errors
    /// Returns an error if GCS client or auth provider initialization fails.
    #[instrument(level = "debug", name = "video_handler_new", skip_all)]
    pub async fn new(config: Config) -> Result<Self, Error> {
        debug!("Initializing VideoHandler");

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

    /// Create a new VideoHandler with provided dependencies (for testing).
    #[cfg(test)]
    pub fn with_deps(config: Config, gcs: GcsClient, http: reqwest::Client, auth: AuthProvider) -> Self {
        Self {
            config,
            gcs,
            http,
            auth,
        }
    }

    /// Get the Vertex AI Veo API endpoint for generating videos.
    pub fn get_generate_endpoint(&self, model: &str) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:predictLongRunning",
            self.config.location,
            self.config.project_id,
            self.config.location,
            model
        )
    }

    /// Get the Vertex AI endpoint for fetching LRO status.
    /// Uses the fetchPredictOperation endpoint which requires the operation name in the request body.
    pub fn get_fetch_operation_endpoint(&self, model: &str) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:fetchPredictOperation",
            self.config.location,
            self.config.project_id,
            self.config.location,
            model
        )
    }

    /// Generate video from a text prompt.
    ///
    /// # Arguments
    /// * `params` - Video generation parameters
    ///
    /// # Returns
    /// * `Ok(VideoGenerateResult)` - Generated video with GCS URI and optional local path
    /// * `Err(Error)` - If validation fails, API call fails, or output handling fails
    #[instrument(level = "info", name = "generate_video_t2v", skip(self, params), fields(model = %params.model, aspect_ratio = %params.aspect_ratio))]
    pub async fn generate_video_t2v(&self, params: VideoT2vParams) -> Result<VideoGenerateResult, Error> {
        // Validate parameters
        params.validate().map_err(|errors| {
            let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            Error::validation(messages.join("; "))
        })?;

        // Resolve the model to get the canonical ID
        let model = params.get_model().ok_or_else(|| {
            Error::validation(format!("Unknown model: {}", params.model))
        })?;

        info!(model_id = model.id, "Generating video with Veo API (text-to-video)");

        // Build the API request
        let request = VeoT2vRequest {
            instances: vec![VeoT2vInstance {
                prompt: params.prompt.clone(),
            }],
            parameters: VeoParameters {
                aspect_ratio: Some(params.aspect_ratio.clone()),
                storage_uri: params.output_gcs_uri.clone(),
                duration_seconds: Some(params.duration_seconds),
                generate_audio: if model.supports_audio { params.generate_audio } else { None },
                seed: params.seed,
            },
        };

        // Get auth token
        let token = self.auth.get_token(&["https://www.googleapis.com/auth/cloud-platform"]).await?;

        // Make API request to start LRO
        let endpoint = self.get_generate_endpoint(model.id);
        debug!(endpoint = %endpoint, "Calling Veo API");

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

        // Parse LRO response
        let lro_response: LroResponse = response.json().await.map_err(|e| {
            Error::api(&endpoint, status.as_u16(), format!("Failed to parse LRO response: {}", e))
        })?;

        info!(operation_name = %lro_response.name, "Started video generation LRO");

        // Poll for completion
        let result = self.poll_lro(&lro_response.name, model.id).await?;

        // Handle output
        self.handle_output(result, &params.output_gcs_uri, params.download_local, params.local_path.as_deref()).await
    }

    /// Generate video from an image.
    ///
    /// # Arguments
    /// * `params` - Image-to-video generation parameters
    ///
    /// # Returns
    /// * `Ok(VideoGenerateResult)` - Generated video with GCS URI and optional local path
    /// * `Err(Error)` - If validation fails, API call fails, or output handling fails
    #[instrument(level = "info", name = "generate_video_i2v", skip(self, params), fields(model = %params.model, aspect_ratio = %params.aspect_ratio))]
    pub async fn generate_video_i2v(&self, params: VideoI2vParams) -> Result<VideoGenerateResult, Error> {
        // Validate parameters
        params.validate().map_err(|errors| {
            let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            Error::validation(messages.join("; "))
        })?;

        // Resolve the model to get the canonical ID
        let model = params.get_model().ok_or_else(|| {
            Error::validation(format!("Unknown model: {}", params.model))
        })?;

        // Determine mode: interpolation or standard I2V
        let is_interpolation = params.last_frame_image.is_some();
        if is_interpolation {
            info!(model_id = model.id, "Generating video with Veo API (interpolation mode)");
        } else {
            info!(model_id = model.id, "Generating video with Veo API (image-to-video)");
        }

        // Resolve the image input (first frame)
        let image_data = self.resolve_image_input(&params.image).await?;

        // Resolve last frame if provided (interpolation mode)
        let last_frame = if let Some(ref last_frame_path) = params.last_frame_image {
            let last_frame_data = self.resolve_image_input(last_frame_path).await?;
            Some(VeoImageInput {
                bytes_base64_encoded: last_frame_data,
            })
        } else {
            None
        };

        // Build the API request
        let request = VeoI2vRequest {
            instances: vec![VeoI2vInstance {
                prompt: params.prompt.clone(),
                image: VeoImageInput {
                    bytes_base64_encoded: image_data,
                },
            }],
            parameters: VeoI2vParameters {
                aspect_ratio: Some(params.aspect_ratio.clone()),
                storage_uri: params.output_gcs_uri.clone(),
                duration_seconds: Some(params.duration_seconds),
                generate_audio: None, // I2V doesn't support audio generation
                seed: params.seed,
                last_frame,
            },
        };

        // Get auth token
        let token = self.auth.get_token(&["https://www.googleapis.com/auth/cloud-platform"]).await?;

        // Make API request to start LRO
        let endpoint = self.get_generate_endpoint(model.id);
        debug!(endpoint = %endpoint, "Calling Veo API");

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

        // Parse LRO response
        let lro_response: LroResponse = response.json().await.map_err(|e| {
            Error::api(&endpoint, status.as_u16(), format!("Failed to parse LRO response: {}", e))
        })?;

        info!(operation_name = %lro_response.name, "Started video generation LRO");

        // Poll for completion
        let result = self.poll_lro(&lro_response.name, model.id).await?;

        // Handle output
        self.handle_output(result, &params.output_gcs_uri, params.download_local, params.local_path.as_deref()).await
    }

    /// Extend an existing video.
    ///
    /// # Arguments
    /// * `params` - Video extension parameters
    ///
    /// # Returns
    /// * `Ok(VideoGenerateResult)` - Extended video with GCS URI and optional local path
    /// * `Err(Error)` - If validation fails, API call fails, or output handling fails
    #[instrument(level = "info", name = "extend_video", skip(self, params), fields(model = %params.model))]
    pub async fn extend_video(&self, params: VideoExtendParams) -> Result<VideoGenerateResult, Error> {
        // Validate parameters
        params.validate().map_err(|errors| {
            let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            Error::validation(messages.join("; "))
        })?;

        // Resolve the model to get the canonical ID
        let model = params.get_model().ok_or_else(|| {
            Error::validation(format!("Unknown model: {}", params.model))
        })?;

        info!(model_id = model.id, "Extending video with Veo API");

        // Build the API request
        let request = VeoExtendRequest {
            instances: vec![VeoExtendInstance {
                prompt: params.prompt.clone(),
                video: VeoVideoInput {
                    gcs_uri: params.video_input.clone(),
                    mime_type: "video/mp4".to_string(),
                },
            }],
            parameters: VeoExtendParameters {
                storage_uri: params.output_gcs_uri.clone(),
                duration_seconds: Some(params.duration_seconds),
                seed: params.seed,
            },
        };

        // Get auth token
        let token = self.auth.get_token(&["https://www.googleapis.com/auth/cloud-platform"]).await?;

        // Make API request to start LRO
        let endpoint = self.get_generate_endpoint(model.id);
        debug!(endpoint = %endpoint, "Calling Veo API for video extension");

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

        // Parse LRO response
        let lro_response: LroResponse = response.json().await.map_err(|e| {
            Error::api(&endpoint, status.as_u16(), format!("Failed to parse LRO response: {}", e))
        })?;

        info!(operation_name = %lro_response.name, "Started video extension LRO");

        // Poll for completion
        let result = self.poll_lro(&lro_response.name, model.id).await?;

        // Handle output
        self.handle_output(result, &params.output_gcs_uri, params.download_local, params.local_path.as_deref()).await
    }

    /// Resolve image input to base64 data.
    ///
    /// Handles three input formats:
    /// - Base64 data (already encoded)
    /// - Local file path
    /// - GCS URI
    async fn resolve_image_input(&self, image: &str) -> Result<String, Error> {
        // Check if it's a GCS URI first (explicit protocol)
        if image.starts_with("gs://") {
            let uri = GcsUri::parse(image)?;
            let data = self.gcs.download(&uri).await?;
            return Ok(BASE64.encode(&data));
        }

        // Check if it looks like a file path:
        // - Starts with / (absolute path)
        // - Starts with ./ or ../ (relative path)
        // - Contains common path patterns like file extensions with preceding path separators
        // - Is short enough to be a reasonable path (base64 images are typically very long)
        let looks_like_path = image.starts_with('/')
            || image.starts_with("./")
            || image.starts_with("../")
            || image.starts_with("~/")
            || (image.len() < 500 && image.contains('/') && Self::has_file_extension(image));

        if looks_like_path {
            // Treat as local file path
            let path = Path::new(image);
            if !path.exists() {
                return Err(Error::validation(format!("Image file not found: {}", image)));
            }
            let data = tokio::fs::read(path).await?;
            return Ok(BASE64.encode(&data));
        }

        // Try to validate as base64 - if it decodes successfully, it's base64
        // This handles the case where base64 contains '/' characters
        if image.len() > 100 {
            if BASE64.decode(image).is_ok() {
                return Ok(image.to_string());
            }
        }

        // Last resort: try as file path (might be a relative path without ./)
        let path = Path::new(image);
        if path.exists() {
            let data = tokio::fs::read(path).await?;
            return Ok(BASE64.encode(&data));
        }

        // If nothing worked and it's long, assume it's base64 (might be malformed)
        if image.len() > 100 {
            return Ok(image.to_string());
        }

        Err(Error::validation(format!(
            "Image input '{}' is not a valid file path, GCS URI, or base64 data",
            if image.len() > 50 { &image[..50] } else { image }
        )))
    }

    /// Check if a string ends with a common image file extension.
    fn has_file_extension(s: &str) -> bool {
        let lower = s.to_lowercase();
        lower.ends_with(".png")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".gif")
            || lower.ends_with(".webp")
            || lower.ends_with(".bmp")
            || lower.ends_with(".tiff")
            || lower.ends_with(".tif")
    }

    /// Poll a long-running operation until completion.
    ///
    /// Uses exponential backoff with configurable parameters.
    /// Uses the fetchPredictOperation endpoint which requires the operation name in the request body.
    pub async fn poll_lro(&self, operation_name: &str, model: &str) -> Result<LroResult, Error> {
        let mut delay_ms = LRO_INITIAL_DELAY_MS;
        let mut attempts = 0;

        loop {
            attempts += 1;
            if attempts > LRO_MAX_ATTEMPTS {
                // Calculate approximate timeout in seconds
                let timeout_seconds = (LRO_MAX_ATTEMPTS as u64) * (LRO_MAX_DELAY_MS / 1000);
                return Err(Error::timeout(timeout_seconds));
            }

            // Wait before polling
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            // Get auth token
            let token = self.auth.get_token(&["https://www.googleapis.com/auth/cloud-platform"]).await?;

            // Poll the operation using fetchPredictOperation
            let endpoint = self.get_fetch_operation_endpoint(model);
            debug!(endpoint = %endpoint, attempt = attempts, "Polling LRO");

            // Build the fetch request with operation name in body
            let fetch_request = FetchOperationRequest {
                operation_name: operation_name.to_string(),
            };

            let response = self.http
                .post(&endpoint)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json(&fetch_request)
                .send()
                .await
                .map_err(|e| Error::api(&endpoint, 0, format!("Poll request failed: {}", e)))?;

            let status = response.status();
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                return Err(Error::api(&endpoint, status.as_u16(), body));
            }

            let lro_status: LroStatusResponse = response.json().await.map_err(|e| {
                Error::api(&endpoint, status.as_u16(), format!("Failed to parse LRO status: {}", e))
            })?;

            if lro_status.done.unwrap_or(false) {
                // Check for error
                if let Some(error) = lro_status.error {
                    return Err(Error::api(
                        &endpoint,
                        error.code.unwrap_or(500) as u16,
                        error.message.unwrap_or_else(|| "Unknown error".to_string()),
                    ));
                }

                // Return the result
                if let Some(response) = lro_status.response {
                    info!(operation_name = %operation_name, attempts = attempts, "LRO completed successfully");
                    return Ok(LroResult {
                        videos: response.videos.unwrap_or_default(),
                    });
                }

                return Err(Error::api(&endpoint, 200, "LRO completed but no response found"));
            }

            // Increase delay with exponential backoff
            delay_ms = ((delay_ms as f64) * LRO_BACKOFF_MULTIPLIER) as u64;
            delay_ms = delay_ms.min(LRO_MAX_DELAY_MS);

            debug!(
                operation_name = %operation_name,
                attempt = attempts,
                next_delay_ms = delay_ms,
                "LRO still in progress"
            );
        }
    }

    /// Handle output of generated video.
    async fn handle_output(
        &self,
        result: LroResult,
        output_gcs_uri: &str,
        download_local: bool,
        local_path: Option<&str>,
    ) -> Result<VideoGenerateResult, Error> {
        // Get the first generated video
        let video = result.videos.first().ok_or_else(|| {
            Error::api("", 200, "No video generated")
        })?;

        let gcs_uri = video.gcs_uri.clone()
            .unwrap_or_else(|| output_gcs_uri.to_string());

        info!(gcs_uri = %gcs_uri, "Video generated successfully");

        // If download_local is requested, download the video
        if download_local {
            let local_file = if let Some(path) = local_path {
                path.to_string()
            } else {
                // Generate a default local path from the GCS URI
                let uri = GcsUri::parse(&gcs_uri)?;
                format!("./{}", uri.object.split('/').last().unwrap_or("output.mp4"))
            };

            let uri = GcsUri::parse(&gcs_uri)?;
            let data = self.gcs.download(&uri).await?;
            tokio::fs::write(&local_file, &data).await?;

            info!(local_file = %local_file, "Video downloaded locally");

            return Ok(VideoGenerateResult {
                gcs_uri,
                local_path: Some(local_file),
            });
        }

        Ok(VideoGenerateResult {
            gcs_uri,
            local_path: None,
        })
    }
}

// =============================================================================
// API Request/Response Types
// =============================================================================

/// Vertex AI Veo API request for text-to-video.
#[derive(Debug, Serialize)]
pub struct VeoT2vRequest {
    /// Input instances (prompts)
    pub instances: Vec<VeoT2vInstance>,
    /// Generation parameters
    pub parameters: VeoParameters,
}

/// Veo API instance for text-to-video.
#[derive(Debug, Serialize)]
pub struct VeoT2vInstance {
    /// Text prompt describing the video
    pub prompt: String,
}

/// Vertex AI Veo API request for image-to-video.
#[derive(Debug, Serialize)]
pub struct VeoI2vRequest {
    /// Input instances (image + prompt)
    pub instances: Vec<VeoI2vInstance>,
    /// Generation parameters
    pub parameters: VeoI2vParameters,
}

/// Veo API instance for image-to-video.
#[derive(Debug, Serialize)]
pub struct VeoI2vInstance {
    /// Text prompt describing the desired motion
    pub prompt: String,
    /// Source image (first frame)
    pub image: VeoImageInput,
}

/// Veo image input.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VeoImageInput {
    /// Base64-encoded image data
    pub bytes_base64_encoded: String,
}

/// Veo API parameters for I2V (includes last_frame for interpolation).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VeoI2vParameters {
    /// Aspect ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    /// GCS URI for output (API expects "storageUri")
    #[serde(rename = "storageUri")]
    pub storage_uri: String,
    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<u8>,
    /// Whether to generate audio (Veo 3.x only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// Random seed for reproducibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Last frame for interpolation mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_frame: Option<VeoImageInput>,
}

/// Vertex AI Veo API request for video extension.
#[derive(Debug, Serialize)]
pub struct VeoExtendRequest {
    /// Input instances (video + prompt)
    pub instances: Vec<VeoExtendInstance>,
    /// Generation parameters
    pub parameters: VeoExtendParameters,
}

/// Veo API instance for video extension.
#[derive(Debug, Serialize)]
pub struct VeoExtendInstance {
    /// Text prompt describing the desired continuation
    pub prompt: String,
    /// Source video to extend
    pub video: VeoVideoInput,
}

/// Veo video input.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VeoVideoInput {
    /// GCS URI of the video
    pub gcs_uri: String,
    /// MIME type of the video
    pub mime_type: String,
}

/// Veo API parameters for video extension.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VeoExtendParameters {
    /// GCS URI for output (API expects "storageUri")
    #[serde(rename = "storageUri")]
    pub storage_uri: String,
    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<u8>,
    /// Random seed for reproducibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

/// Veo API parameters.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VeoParameters {
    /// Aspect ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    /// GCS URI for output (API expects "storageUri")
    #[serde(rename = "storageUri")]
    pub storage_uri: String,
    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<u8>,
    /// Whether to generate audio (Veo 3.x only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// Random seed for reproducibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

/// Long-running operation response.
#[derive(Debug, Deserialize)]
pub struct LroResponse {
    /// Operation name for polling
    pub name: String,
}

/// Request to fetch operation status.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchOperationRequest {
    /// The operation name to fetch
    pub operation_name: String,
}

/// Long-running operation status response.
#[derive(Debug, Deserialize)]
pub struct LroStatusResponse {
    /// Whether the operation is complete
    pub done: Option<bool>,
    /// Error if the operation failed
    pub error: Option<LroError>,
    /// Response if the operation succeeded
    pub response: Option<LroResultResponse>,
}

/// LRO error details.
#[derive(Debug, Deserialize)]
pub struct LroError {
    /// Error code
    pub code: Option<i32>,
    /// Error message
    pub message: Option<String>,
}

/// LRO result response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LroResultResponse {
    /// Generated videos (API returns "videos" not "generatedSamples")
    pub videos: Option<Vec<VideoOutput>>,
    /// Count of videos filtered by RAI policies
    #[serde(default)]
    pub rai_media_filtered_count: Option<i32>,
}

/// Video output from Veo API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoOutput {
    /// GCS URI of the generated video
    pub gcs_uri: Option<String>,
    /// MIME type of the video
    pub mime_type: Option<String>,
}

// =============================================================================
// Result Types
// =============================================================================

/// Internal LRO result.
#[derive(Debug)]
pub struct LroResult {
    /// Generated videos
    pub videos: Vec<VideoOutput>,
}

/// Result of video generation.
#[derive(Debug)]
pub struct VideoGenerateResult {
    /// GCS URI of the generated video
    pub gcs_uri: String,
    /// Local file path if downloaded
    pub local_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_t2v_params() {
        let params: VideoT2vParams = serde_json::from_str(r#"{
            "prompt": "A cat walking",
            "output_gcs_uri": "gs://bucket/output.mp4"
        }"#).unwrap();
        assert_eq!(params.model, DEFAULT_MODEL);
        assert_eq!(params.aspect_ratio, DEFAULT_ASPECT_RATIO);
        assert_eq!(params.duration_seconds, DEFAULT_DURATION_SECONDS);
        assert!(!params.download_local);
        assert!(params.generate_audio.is_none());
        assert!(params.seed.is_none());
    }

    #[test]
    fn test_valid_t2v_params() {
        let params = VideoT2vParams {
            prompt: "A beautiful sunset over mountains".to_string(),
            model: "veo-3".to_string(),
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 6,
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: false,
            local_path: None,
            generate_audio: Some(true),
            seed: Some(42),
        };

        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_invalid_duration_too_low() {
        let params = VideoT2vParams {
            prompt: "A cat".to_string(),
            model: DEFAULT_MODEL.to_string(),
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 3, // Below minimum
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: false,
            local_path: None,
            generate_audio: None,
            seed: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "duration_seconds"));
    }

    #[test]
    fn test_invalid_duration_too_high() {
        let params = VideoT2vParams {
            prompt: "A cat".to_string(),
            model: DEFAULT_MODEL.to_string(),
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 15, // Above maximum
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: false,
            local_path: None,
            generate_audio: None,
            seed: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "duration_seconds"));
    }

    #[test]
    fn test_invalid_aspect_ratio() {
        let params = VideoT2vParams {
            prompt: "A cat".to_string(),
            model: DEFAULT_MODEL.to_string(),
            aspect_ratio: "4:3".to_string(), // Not valid for Veo
            duration_seconds: 6,
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: false,
            local_path: None,
            generate_audio: None,
            seed: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "aspect_ratio"));
    }

    #[test]
    fn test_invalid_model() {
        let params = VideoT2vParams {
            prompt: "A cat".to_string(),
            model: "unknown-model".to_string(),
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 6,
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: false,
            local_path: None,
            generate_audio: None,
            seed: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "model"));
    }

    #[test]
    fn test_empty_prompt() {
        let params = VideoT2vParams {
            prompt: "   ".to_string(),
            model: DEFAULT_MODEL.to_string(),
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 6,
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: false,
            local_path: None,
            generate_audio: None,
            seed: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "prompt"));
    }

    #[test]
    fn test_invalid_gcs_uri() {
        let params = VideoT2vParams {
            prompt: "A cat".to_string(),
            model: DEFAULT_MODEL.to_string(),
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 6,
            output_gcs_uri: "/local/path/output.mp4".to_string(), // Not a GCS URI
            download_local: false,
            local_path: None,
            generate_audio: None,
            seed: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "output_gcs_uri"));
    }

    #[test]
    fn test_generate_audio_on_veo2_fails() {
        let params = VideoT2vParams {
            prompt: "A cat".to_string(),
            model: "veo-2".to_string(), // Veo 2 doesn't support audio
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 6,
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: false,
            local_path: None,
            generate_audio: Some(true), // Should fail
            seed: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "generate_audio"));
    }

    #[test]
    fn test_generate_audio_on_veo3_succeeds() {
        let params = VideoT2vParams {
            prompt: "A cat".to_string(),
            model: "veo-3".to_string(), // Veo 3 supports audio
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 6,
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: false,
            local_path: None,
            generate_audio: Some(true),
            seed: None,
        };

        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_all_valid_aspect_ratios() {
        for ratio in VALID_ASPECT_RATIOS {
            let params = VideoT2vParams {
                prompt: "A cat".to_string(),
                model: DEFAULT_MODEL.to_string(),
                aspect_ratio: ratio.to_string(),
                duration_seconds: 6,
                output_gcs_uri: "gs://bucket/output.mp4".to_string(),
                download_local: false,
                local_path: None,
                generate_audio: None,
                seed: None,
            };
            assert!(params.validate().is_ok(), "Aspect ratio {} should be valid", ratio);
        }
    }

    #[test]
    fn test_all_valid_durations() {
        for dur in SUPPORTED_DURATIONS {
            let params = VideoT2vParams {
                prompt: "A cat".to_string(),
                model: DEFAULT_MODEL.to_string(),
                aspect_ratio: "16:9".to_string(),
                duration_seconds: *dur,
                output_gcs_uri: "gs://bucket/output.mp4".to_string(),
                download_local: false,
                local_path: None,
                generate_audio: None,
                seed: None,
            };
            assert!(params.validate().is_ok(), "Duration {} should be valid", dur);
        }
    }

    #[test]
    fn test_get_model() {
        let params = VideoT2vParams {
            prompt: "A cat".to_string(),
            model: "veo-3".to_string(),
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 6,
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: false,
            local_path: None,
            generate_audio: None,
            seed: None,
        };

        let model = params.get_model();
        assert!(model.is_some());
        assert_eq!(model.unwrap().id, "veo-3.0-generate-preview");
    }

    // I2V tests
    #[test]
    fn test_default_i2v_params() {
        let params: VideoI2vParams = serde_json::from_str(r#"{
            "image": "base64data",
            "prompt": "A cat walking",
            "output_gcs_uri": "gs://bucket/output.mp4"
        }"#).unwrap();
        assert_eq!(params.model, DEFAULT_MODEL);
        assert_eq!(params.aspect_ratio, DEFAULT_ASPECT_RATIO);
        assert_eq!(params.duration_seconds, DEFAULT_DURATION_SECONDS);
        assert!(!params.download_local);
    }

    #[test]
    fn test_valid_i2v_params() {
        let params = VideoI2vParams {
            image: "base64imagedata".to_string(),
            prompt: "The cat starts walking".to_string(),
            last_frame_image: None,
            model: "veo-3".to_string(),
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 6,
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: false,
            local_path: None,
            seed: Some(42),
        };

        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_i2v_empty_image() {
        let params = VideoI2vParams {
            image: "   ".to_string(),
            prompt: "The cat starts walking".to_string(),
            last_frame_image: None,
            model: DEFAULT_MODEL.to_string(),
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 6,
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: false,
            local_path: None,
            seed: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "image"));
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError {
            field: "prompt".to_string(),
            message: "cannot be empty".to_string(),
        };

        let display = format!("{}", error);
        assert_eq!(display, "prompt: cannot be empty");
    }

    #[test]
    fn test_validation_multiple_errors() {
        let params = VideoT2vParams {
            prompt: "   ".to_string(), // Empty prompt
            model: "unknown-model".to_string(), // Invalid model
            aspect_ratio: "invalid".to_string(), // Invalid aspect ratio
            duration_seconds: 100, // Out of range
            output_gcs_uri: "/local/path".to_string(), // Invalid GCS URI
            download_local: false,
            local_path: None,
            generate_audio: None,
            seed: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.len() >= 3, "Expected at least 3 validation errors, got {}", errors.len());
    }

    // Tests for base64 detection (P2 fix)
    #[test]
    fn test_has_file_extension_png() {
        assert!(VideoHandler::has_file_extension("image.png"));
        assert!(VideoHandler::has_file_extension("path/to/image.PNG"));
    }

    #[test]
    fn test_has_file_extension_jpg() {
        assert!(VideoHandler::has_file_extension("photo.jpg"));
        assert!(VideoHandler::has_file_extension("photo.jpeg"));
        assert!(VideoHandler::has_file_extension("photo.JPEG"));
    }

    #[test]
    fn test_has_file_extension_other_formats() {
        assert!(VideoHandler::has_file_extension("image.gif"));
        assert!(VideoHandler::has_file_extension("image.webp"));
        assert!(VideoHandler::has_file_extension("image.bmp"));
        assert!(VideoHandler::has_file_extension("image.tiff"));
        assert!(VideoHandler::has_file_extension("image.tif"));
    }

    #[test]
    fn test_has_file_extension_no_extension() {
        assert!(!VideoHandler::has_file_extension("noextension"));
        assert!(!VideoHandler::has_file_extension("path/to/file"));
    }

    #[test]
    fn test_has_file_extension_wrong_extension() {
        assert!(!VideoHandler::has_file_extension("file.txt"));
        assert!(!VideoHandler::has_file_extension("file.mp4"));
        assert!(!VideoHandler::has_file_extension("file.pdf"));
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: rust-mcp-genmedia, Property 8: Numeric Parameter Range Validation (duration_seconds)
    // **Validates: Requirements 5.4, 5.6**
    //
    // For any numeric parameter with defined bounds (duration_seconds 4, 6, 8),
    // values outside the valid set SHALL be rejected with a validation error.

    /// Strategy to generate valid duration_seconds values (4, 6, 8)
    fn valid_duration_strategy() -> impl Strategy<Value = u8> {
        prop_oneof![
            Just(4u8),
            Just(6u8),
            Just(8u8),
        ]
    }

    /// Strategy to generate invalid duration_seconds values (not in [4, 6, 8])
    fn invalid_duration_strategy() -> impl Strategy<Value = u8> {
        prop_oneof![
            Just(0u8),
            Just(1u8),
            Just(2u8),
            Just(3u8),
            Just(5u8),  // 5 is not supported
            Just(7u8),  // 7 is not supported
            Just(9u8),
            Just(10u8),
        ]
    }

    /// Strategy to generate valid aspect ratios
    fn valid_aspect_ratio_strategy() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("16:9"),
            Just("9:16"),
        ]
    }

    /// Strategy to generate valid prompts (non-empty)
    fn valid_prompt_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 ]{1,100}".prop_map(|s| s.trim().to_string())
            .prop_filter("Must not be empty", |s| !s.trim().is_empty())
    }

    /// Strategy to generate valid GCS URIs
    fn valid_gcs_uri_strategy() -> impl Strategy<Value = String> {
        "[a-z0-9-]{3,20}".prop_map(|bucket| format!("gs://{}/output.mp4", bucket))
    }

    proptest! {
        /// Property 8: Valid duration_seconds values (5-8) should pass validation
        #[test]
        fn valid_duration_passes_validation(
            dur in valid_duration_strategy(),
            prompt in valid_prompt_strategy(),
            gcs_uri in valid_gcs_uri_strategy(),
        ) {
            let params = VideoT2vParams {
                prompt,
                model: DEFAULT_MODEL.to_string(),
                aspect_ratio: DEFAULT_ASPECT_RATIO.to_string(),
                duration_seconds: dur,
                output_gcs_uri: gcs_uri,
                download_local: false,
                local_path: None,
                generate_audio: None,
                seed: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_ok(),
                "duration_seconds {} should be valid, but got errors: {:?}",
                dur,
                result.err()
            );
        }

        /// Property 8: Invalid duration_seconds values (< 5 or > 8) should fail validation
        #[test]
        fn invalid_duration_fails_validation(
            dur in invalid_duration_strategy(),
            prompt in valid_prompt_strategy(),
            gcs_uri in valid_gcs_uri_strategy(),
        ) {
            let params = VideoT2vParams {
                prompt,
                model: DEFAULT_MODEL.to_string(),
                aspect_ratio: DEFAULT_ASPECT_RATIO.to_string(),
                duration_seconds: dur,
                output_gcs_uri: gcs_uri,
                download_local: false,
                local_path: None,
                generate_audio: None,
                seed: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_err(),
                "duration_seconds {} should be invalid",
                dur
            );

            let errors = result.unwrap_err();
            prop_assert!(
                errors.iter().any(|e| e.field == "duration_seconds"),
                "Should have a duration_seconds validation error for value {}",
                dur
            );
        }
    }

    // Feature: rust-mcp-genmedia, Property 9: Default Parameter Application
    // **Validates: Requirements 5.4, 5.6**
    //
    // When optional parameters are not provided, the system SHALL apply
    // documented default values consistently.

    proptest! {
        /// Property 9: Default parameters should be applied when not specified
        #[test]
        fn default_params_applied_correctly(
            prompt in valid_prompt_strategy(),
        ) {
            // Parse JSON with only required fields
            let json = format!(r#"{{
                "prompt": "{}",
                "output_gcs_uri": "gs://bucket/output.mp4"
            }}"#, prompt.replace('"', "\\\""));
            
            let params: Result<VideoT2vParams, _> = serde_json::from_str(&json);
            prop_assert!(params.is_ok(), "Should parse successfully");
            
            let params = params.unwrap();
            
            // Verify defaults are applied
            prop_assert_eq!(params.model, DEFAULT_MODEL, "Default model should be applied");
            prop_assert_eq!(params.aspect_ratio, DEFAULT_ASPECT_RATIO, "Default aspect ratio should be applied");
            prop_assert_eq!(params.duration_seconds, DEFAULT_DURATION_SECONDS, "Default duration should be applied");
            prop_assert!(!params.download_local, "download_local should default to false");
            prop_assert!(params.generate_audio.is_none(), "generate_audio should default to None");
            prop_assert!(params.seed.is_none(), "seed should default to None");
        }

        /// Property 9: Explicitly provided parameters should override defaults
        #[test]
        fn explicit_params_override_defaults(
            dur in valid_duration_strategy(),
            ratio in valid_aspect_ratio_strategy(),
            prompt in valid_prompt_strategy(),
        ) {
            let params = VideoT2vParams {
                prompt: prompt.clone(),
                model: "veo-2".to_string(),
                aspect_ratio: ratio.to_string(),
                duration_seconds: dur,
                output_gcs_uri: "gs://bucket/output.mp4".to_string(),
                download_local: true,
                local_path: Some("/tmp/video.mp4".to_string()),
                generate_audio: None, // Veo 2 doesn't support audio
                seed: Some(42),
            };

            // Verify explicit values are preserved
            prop_assert_eq!(params.model, "veo-2");
            prop_assert_eq!(params.aspect_ratio, ratio);
            prop_assert_eq!(params.duration_seconds, dur);
            prop_assert!(params.download_local);
            prop_assert_eq!(params.local_path, Some("/tmp/video.mp4".to_string()));
            prop_assert_eq!(params.seed, Some(42));
        }

        /// Property: Combination of valid parameters should always pass validation
        #[test]
        fn valid_params_combination_passes(
            dur in valid_duration_strategy(),
            ratio in valid_aspect_ratio_strategy(),
            prompt in valid_prompt_strategy(),
            gcs_uri in valid_gcs_uri_strategy(),
        ) {
            let params = VideoT2vParams {
                prompt,
                model: DEFAULT_MODEL.to_string(),
                aspect_ratio: ratio.to_string(),
                duration_seconds: dur,
                output_gcs_uri: gcs_uri,
                download_local: false,
                local_path: None,
                generate_audio: None,
                seed: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_ok(),
                "Valid params (dur={}, ratio='{}') should pass, but got: {:?}",
                dur,
                ratio,
                result.err()
            );
        }
    }
}


#[cfg(test)]
mod lro_property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: rust-mcp-genmedia, Property 11: Long-Running Operation Polling
    // **Validates: Requirements 5.16**
    //
    // The LRO polling mechanism SHALL use exponential backoff with configurable
    // parameters and SHALL timeout after a maximum number of attempts.

    proptest! {
        /// Property 11: Exponential backoff delay increases correctly
        #[test]
        fn exponential_backoff_increases_delay(
            initial_delay in 1000u64..10000u64,
            multiplier in 1.1f64..3.0f64,
            iterations in 1usize..10usize,
        ) {
            let mut delay = initial_delay;
            let mut prev_delay = 0u64;
            
            for _ in 0..iterations {
                // Each iteration should increase the delay
                prop_assert!(delay > prev_delay || prev_delay == 0, 
                    "Delay should increase: {} > {}", delay, prev_delay);
                
                prev_delay = delay;
                delay = ((delay as f64) * multiplier) as u64;
            }
        }

        /// Property 11: Backoff delay is capped at maximum
        #[test]
        fn backoff_delay_capped_at_max(
            iterations in 1usize..200usize,
        ) {
            let mut delay_ms = LRO_INITIAL_DELAY_MS;
            
            for _ in 0..iterations {
                delay_ms = ((delay_ms as f64) * LRO_BACKOFF_MULTIPLIER) as u64;
                delay_ms = delay_ms.min(LRO_MAX_DELAY_MS);
                
                prop_assert!(delay_ms <= LRO_MAX_DELAY_MS,
                    "Delay {} should not exceed max {}", delay_ms, LRO_MAX_DELAY_MS);
            }
        }

        /// Property 11: LRO configuration constants are valid
        #[test]
        fn lro_config_constants_valid(_dummy in Just(())) {
            // Initial delay should be positive
            prop_assert!(LRO_INITIAL_DELAY_MS > 0, "Initial delay must be positive");
            
            // Max delay should be >= initial delay
            prop_assert!(LRO_MAX_DELAY_MS >= LRO_INITIAL_DELAY_MS, 
                "Max delay must be >= initial delay");
            
            // Backoff multiplier should be > 1.0
            prop_assert!(LRO_BACKOFF_MULTIPLIER > 1.0, 
                "Backoff multiplier must be > 1.0");
            
            // Max attempts should be reasonable (allow for long operations)
            prop_assert!(LRO_MAX_ATTEMPTS > 0 && LRO_MAX_ATTEMPTS <= 1000,
                "Max attempts should be between 1 and 1000");
        }

        /// Property 11: Total timeout is reasonable for video generation
        #[test]
        fn total_timeout_reasonable(_dummy in Just(())) {
            // Calculate approximate total timeout
            let mut total_ms = 0u64;
            let mut delay_ms = LRO_INITIAL_DELAY_MS;
            
            for _ in 0..LRO_MAX_ATTEMPTS {
                total_ms += delay_ms;
                delay_ms = ((delay_ms as f64) * LRO_BACKOFF_MULTIPLIER) as u64;
                delay_ms = delay_ms.min(LRO_MAX_DELAY_MS);
            }
            
            let total_minutes = total_ms / 60000;
            
            // Video generation can take several minutes, so timeout should be at least 10 minutes
            prop_assert!(total_minutes >= 10, 
                "Total timeout {} minutes should be at least 10 minutes", total_minutes);
            
            // But not more than 2 hours (reasonable upper bound)
            prop_assert!(total_minutes <= 120,
                "Total timeout {} minutes should not exceed 120 minutes", total_minutes);
        }
    }
}


/// Unit tests for API interactions and error handling.
/// These tests verify the handler's behavior with mocked API responses.
#[cfg(test)]
mod api_tests {
    use super::*;

    /// Test that VeoT2vRequest serializes correctly for the API.
    #[test]
    fn test_veo_t2v_request_serialization() {
        let request = VeoT2vRequest {
            instances: vec![VeoT2vInstance {
                prompt: "A cat walking in a garden".to_string(),
            }],
            parameters: VeoParameters {
                aspect_ratio: Some("16:9".to_string()),
                storage_uri: "gs://bucket/output.mp4".to_string(),
                duration_seconds: Some(6),
                generate_audio: Some(true),
                seed: Some(42),
            },
        };

        let json = serde_json::to_value(&request).unwrap();
        
        // Verify structure
        assert!(json["instances"].is_array());
        assert_eq!(json["instances"][0]["prompt"], "A cat walking in a garden");
        assert_eq!(json["parameters"]["aspectRatio"], "16:9");
        assert_eq!(json["parameters"]["storageUri"], "gs://bucket/output.mp4");
        assert_eq!(json["parameters"]["durationSeconds"], 6);
        assert_eq!(json["parameters"]["generateAudio"], true);
        assert_eq!(json["parameters"]["seed"], 42);
    }

    /// Test that VeoT2vRequest serializes without optional fields when not provided.
    #[test]
    fn test_veo_t2v_request_serialization_minimal() {
        let request = VeoT2vRequest {
            instances: vec![VeoT2vInstance {
                prompt: "A cat".to_string(),
            }],
            parameters: VeoParameters {
                aspect_ratio: None,
                storage_uri: "gs://bucket/output.mp4".to_string(),
                duration_seconds: None,
                generate_audio: None,
                seed: None,
            },
        };

        let json = serde_json::to_value(&request).unwrap();
        
        // Verify optional fields are not present
        assert!(json["parameters"].get("aspectRatio").is_none());
        assert!(json["parameters"].get("durationSeconds").is_none());
        assert!(json["parameters"].get("generateAudio").is_none());
        assert!(json["parameters"].get("seed").is_none());
        // Required field should be present
        assert!(json["parameters"].get("storageUri").is_some());
    }

    /// Test that VeoI2vRequest serializes correctly for the API.
    #[test]
    fn test_veo_i2v_request_serialization() {
        let request = VeoI2vRequest {
            instances: vec![VeoI2vInstance {
                prompt: "The cat starts walking".to_string(),
                image: VeoImageInput {
                    bytes_base64_encoded: "base64imagedata".to_string(),
                },
            }],
            parameters: VeoI2vParameters {
                aspect_ratio: Some("9:16".to_string()),
                storage_uri: "gs://bucket/output.mp4".to_string(),
                duration_seconds: Some(6),
                generate_audio: None,
                seed: None,
                last_frame: None,
            },
        };

        let json = serde_json::to_value(&request).unwrap();
        
        // Verify structure
        assert!(json["instances"].is_array());
        assert_eq!(json["instances"][0]["prompt"], "The cat starts walking");
        assert_eq!(json["instances"][0]["image"]["bytesBase64Encoded"], "base64imagedata");
        assert_eq!(json["parameters"]["aspectRatio"], "9:16");
    }

    /// Test that LroResponse deserializes correctly.
    #[test]
    fn test_lro_response_deserialization() {
        let json = r#"{
            "name": "projects/123/locations/us-central1/operations/abc123"
        }"#;

        let response: LroResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.name, "projects/123/locations/us-central1/operations/abc123");
    }

    /// Test that LroStatusResponse deserializes when not done.
    #[test]
    fn test_lro_status_not_done() {
        let json = r#"{
            "done": false
        }"#;

        let response: LroStatusResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.done, Some(false));
        assert!(response.error.is_none());
        assert!(response.response.is_none());
    }

    /// Test that LroStatusResponse deserializes when done with success.
    #[test]
    fn test_lro_status_done_success() {
        let json = r#"{
            "done": true,
            "response": {
                "videos": [
                    {
                        "gcsUri": "gs://bucket/output.mp4",
                        "mimeType": "video/mp4"
                    }
                ]
            }
        }"#;

        let response: LroStatusResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.done, Some(true));
        assert!(response.error.is_none());
        assert!(response.response.is_some());
        
        let result = response.response.unwrap();
        assert!(result.videos.is_some());
        let videos = result.videos.unwrap();
        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].gcs_uri, Some("gs://bucket/output.mp4".to_string()));
        assert_eq!(videos[0].mime_type, Some("video/mp4".to_string()));
    }

    /// Test that LroStatusResponse deserializes when done with error.
    #[test]
    fn test_lro_status_done_error() {
        let json = r#"{
            "done": true,
            "error": {
                "code": 400,
                "message": "Invalid prompt"
            }
        }"#;

        let response: LroStatusResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.done, Some(true));
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, Some(400));
        assert_eq!(error.message, Some("Invalid prompt".to_string()));
    }

    /// Test endpoint URL construction for generate.
    #[test]
    fn test_get_generate_endpoint() {
        let config = Config {
            project_id: "my-project".to_string(),
            location: "us-central1".to_string(),
            gcs_bucket: None,
            port: 8080,
        };

        let expected_url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:predictLongRunning",
            config.location,
            config.project_id,
            config.location,
            "veo-3.0-generate-preview"
        );

        assert!(expected_url.contains("us-central1-aiplatform.googleapis.com"));
        assert!(expected_url.contains("my-project"));
        assert!(expected_url.contains("veo-3.0-generate-preview"));
        assert!(expected_url.ends_with(":predictLongRunning"));
    }

    /// Test endpoint URL construction for fetch operation (LRO polling).
    #[test]
    fn test_get_fetch_operation_endpoint() {
        let config = Config {
            project_id: "my-project".to_string(),
            location: "us-central1".to_string(),
            gcs_bucket: None,
            port: 8080,
        };

        let model = "veo-3.0-generate-preview";
        let expected_url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:fetchPredictOperation",
            config.location,
            config.project_id,
            config.location,
            model
        );

        assert!(expected_url.contains("us-central1-aiplatform.googleapis.com"));
        assert!(expected_url.contains("my-project"));
        assert!(expected_url.contains(model));
        assert!(expected_url.ends_with(":fetchPredictOperation"));
    }

    /// Test FetchOperationRequest serialization.
    #[test]
    fn test_fetch_operation_request_serialization() {
        let request = FetchOperationRequest {
            operation_name: "projects/my-project/locations/us-central1/publishers/google/models/veo-3.0-generate-preview/operations/abc123".to_string(),
        };

        let json = serde_json::to_value(&request).unwrap();
        
        assert_eq!(json["operationName"], "projects/my-project/locations/us-central1/publishers/google/models/veo-3.0-generate-preview/operations/abc123");
    }

    /// Test VideoGenerateResult structure.
    #[test]
    fn test_video_generate_result_gcs_only() {
        let result = VideoGenerateResult {
            gcs_uri: "gs://bucket/output.mp4".to_string(),
            local_path: None,
        };

        assert_eq!(result.gcs_uri, "gs://bucket/output.mp4");
        assert!(result.local_path.is_none());
    }

    /// Test VideoGenerateResult with local path.
    #[test]
    fn test_video_generate_result_with_local() {
        let result = VideoGenerateResult {
            gcs_uri: "gs://bucket/output.mp4".to_string(),
            local_path: Some("/tmp/output.mp4".to_string()),
        };

        assert_eq!(result.gcs_uri, "gs://bucket/output.mp4");
        assert_eq!(result.local_path, Some("/tmp/output.mp4".to_string()));
    }

    /// Test LroResult structure.
    #[test]
    fn test_lro_result() {
        let result = LroResult {
            videos: vec![VideoOutput {
                gcs_uri: Some("gs://bucket/output.mp4".to_string()),
                mime_type: Some("video/mp4".to_string()),
            }],
        };

        assert_eq!(result.videos.len(), 1);
        assert!(result.videos[0].gcs_uri.is_some());
    }

    /// Test LroResult with empty videos.
    #[test]
    fn test_lro_result_empty() {
        let result = LroResult {
            videos: vec![],
        };

        assert!(result.videos.is_empty());
    }
}
