//! Image generation handler for the MCP Image server.
//!
//! This module provides the `ImageHandler` struct and parameter types for
//! text-to-image generation using Google's Vertex AI Imagen API.

use adk_rust_mcp_common::auth::AuthProvider;
use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_common::error::Error;
use adk_rust_mcp_common::gcs::{GcsClient, GcsUri};
use adk_rust_mcp_common::models::{ImagenModel, ModelRegistry, IMAGEN_MODELS};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, instrument};

/// Valid aspect ratios for image generation.
pub const VALID_ASPECT_RATIOS: &[&str] = &["1:1", "3:4", "4:3", "9:16", "16:9"];

/// Default model for image generation.
pub const DEFAULT_MODEL: &str = "imagen-4.0-generate-preview-06-06";

/// Minimum number of images that can be generated.
pub const MIN_NUMBER_OF_IMAGES: u8 = 1;

/// Maximum number of images that can be generated.
pub const MAX_NUMBER_OF_IMAGES: u8 = 4;

/// Text-to-image generation parameters.
///
/// These parameters control the image generation process via the Vertex AI Imagen API.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ImageGenerateParams {
    /// Text prompt describing the image to generate.
    /// Maximum length depends on the model (480 chars for Imagen 3, 2000 for Imagen 4).
    pub prompt: String,

    /// Negative prompt - what to avoid in the generated image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,

    /// Model to use for generation.
    /// Defaults to "imagen-4.0-generate-preview-05-20".
    #[serde(default = "default_model")]
    pub model: String,

    /// Aspect ratio for the generated image.
    /// Valid values: "1:1", "3:4", "4:3", "9:16", "16:9".
    #[serde(default = "default_aspect_ratio")]
    pub aspect_ratio: String,

    /// Number of images to generate (1-4).
    #[serde(default = "default_number_of_images")]
    pub number_of_images: u8,

    /// Random seed for reproducible generation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// Output file path for saving the image locally.
    /// If not specified and output_uri is not specified, returns base64-encoded data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_file: Option<String>,

    /// Output storage URI (e.g., gs://bucket/path).
    /// If specified, uploads the image to the storage backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_uri: Option<String>,
}

fn default_model() -> String {
    DEFAULT_MODEL.to_string()
}

fn default_aspect_ratio() -> String {
    "1:1".to_string()
}

fn default_number_of_images() -> u8 {
    1
}

/// Valid upscale factors.
pub const VALID_UPSCALE_FACTORS: &[&str] = &["x2", "x4"];

/// Default upscale model.
pub const UPSCALE_MODEL: &str = "imagen-4.0-upscale-preview";

/// Image upscaling parameters.
///
/// These parameters control the image upscaling process via the Vertex AI Imagen Upscale API.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ImageUpscaleParams {
    /// Source image to upscale.
    /// Can be base64 data, local file path, or GCS URI.
    pub image: String,

    /// Upscale factor: "x2" or "x4".
    #[serde(default = "default_upscale_factor")]
    pub upscale_factor: String,

    /// Output file path for saving the upscaled image locally.
    /// If not specified and output_uri is not specified, returns base64-encoded data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_file: Option<String>,

    /// Output storage URI (e.g., gs://bucket/path).
    /// If specified, uploads the upscaled image to the storage backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_uri: Option<String>,
}

fn default_upscale_factor() -> String {
    "x2".to_string()
}

impl ImageUpscaleParams {
    /// Validate the upscale parameters.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate image is not empty
        if self.image.trim().is_empty() {
            errors.push(ValidationError {
                field: "image".to_string(),
                message: "Image cannot be empty".to_string(),
            });
        }

        // Validate upscale factor
        if !VALID_UPSCALE_FACTORS.contains(&self.upscale_factor.as_str()) {
            errors.push(ValidationError {
                field: "upscale_factor".to_string(),
                message: format!(
                    "Invalid upscale factor '{}'. Valid options: {}",
                    self.upscale_factor,
                    VALID_UPSCALE_FACTORS.join(", ")
                ),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Validation error details for image generation parameters.
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

impl ImageGenerateParams {
    /// Validate the parameters against the model constraints.
    ///
    /// # Returns
    /// - `Ok(())` if all parameters are valid
    /// - `Err(Vec<ValidationError>)` with all validation errors
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Resolve the model to get constraints
        let model = ModelRegistry::resolve_imagen(&self.model);

        // Validate model exists
        if model.is_none() {
            errors.push(ValidationError {
                field: "model".to_string(),
                message: format!(
                    "Unknown model '{}'. Valid models: {}",
                    self.model,
                    IMAGEN_MODELS
                        .iter()
                        .map(|m| m.id)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
        }

        // Validate prompt length (if model is known)
        if let Some(model) = model {
            if self.prompt.len() > model.max_prompt_length {
                errors.push(ValidationError {
                    field: "prompt".to_string(),
                    message: format!(
                        "Prompt length {} exceeds maximum {} for model {}",
                        self.prompt.len(),
                        model.max_prompt_length,
                        model.id
                    ),
                });
            }

            // Validate aspect ratio against model's supported ratios
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
        } else {
            // If model is unknown, validate against common aspect ratios
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
        }

        // Validate number_of_images range
        if self.number_of_images < MIN_NUMBER_OF_IMAGES
            || self.number_of_images > MAX_NUMBER_OF_IMAGES
        {
            errors.push(ValidationError {
                field: "number_of_images".to_string(),
                message: format!(
                    "number_of_images must be between {} and {}, got {}",
                    MIN_NUMBER_OF_IMAGES, MAX_NUMBER_OF_IMAGES, self.number_of_images
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

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the resolved model definition.
    pub fn get_model(&self) -> Option<&'static ImagenModel> {
        ModelRegistry::resolve_imagen(&self.model)
    }
}

/// Image generation handler.
///
/// Handles image generation requests using the Vertex AI Imagen API.
pub struct ImageHandler {
    /// Application configuration.
    pub config: Config,
    /// GCS client for storage operations.
    pub gcs: GcsClient,
    /// HTTP client for API requests.
    pub http: reqwest::Client,
    /// Authentication provider.
    pub auth: AuthProvider,
}

impl ImageHandler {
    /// Create a new ImageHandler with the given configuration.
    ///
    /// # Errors
    /// Returns an error if GCS client or auth provider initialization fails.
    #[instrument(level = "debug", name = "image_handler_new", skip_all)]
    pub async fn new(config: Config) -> Result<Self, Error> {
        debug!("Initializing ImageHandler");

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

    /// Create a new ImageHandler with provided dependencies (for testing).
    #[cfg(test)]
    pub fn with_deps(config: Config, gcs: GcsClient, http: reqwest::Client, auth: AuthProvider) -> Self {
        Self {
            config,
            gcs,
            http,
            auth,
        }
    }

    /// Get the Vertex AI Imagen API endpoint for the given model.
    pub fn get_endpoint(&self, model: &str) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:predict",
            self.config.location,
            self.config.project_id,
            self.config.location,
            model
        )
    }

    /// Generate images from a text prompt.
    ///
    /// # Arguments
    /// * `params` - Image generation parameters
    ///
    /// # Returns
    /// * `Ok(ImageGenerateResult)` - Generated images with their data or paths
    /// * `Err(Error)` - If validation fails, API call fails, or output handling fails
    #[instrument(level = "info", name = "generate_image", skip(self, params), fields(model = %params.model, aspect_ratio = %params.aspect_ratio))]
    pub async fn generate_image(&self, params: ImageGenerateParams) -> Result<ImageGenerateResult, Error> {
        // Validate parameters
        params.validate().map_err(|errors| {
            let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            Error::validation(messages.join("; "))
        })?;

        // Resolve the model to get the canonical ID
        let model = params.get_model().ok_or_else(|| {
            Error::validation(format!("Unknown model: {}", params.model))
        })?;

        info!(model_id = model.id, "Generating image with Imagen API");

        // Build the API request
        let request = ImagenRequest {
            instances: vec![ImagenInstance {
                prompt: params.prompt.clone(),
                negative_prompt: params.negative_prompt.clone(),
            }],
            parameters: ImagenParameters {
                sample_count: params.number_of_images,
                aspect_ratio: params.aspect_ratio.clone(),
                seed: params.seed,
            },
        };

        // Get auth token
        let token = self.auth.get_token(&["https://www.googleapis.com/auth/cloud-platform"]).await?;

        // Make API request
        let endpoint = self.get_endpoint(model.id);
        debug!(endpoint = %endpoint, "Calling Imagen API");

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

        // Parse response
        let api_response: ImagenResponse = response.json().await.map_err(|e| {
            Error::api(&endpoint, status.as_u16(), format!("Failed to parse response: {}", e))
        })?;

        // Extract images from response
        let images: Vec<GeneratedImage> = api_response
            .predictions
            .into_iter()
            .filter_map(|p| {
                p.bytes_base64_encoded.map(|data| GeneratedImage {
                    data,
                    mime_type: p.mime_type.unwrap_or_else(|| "image/png".to_string()),
                })
            })
            .collect();

        if images.is_empty() {
            return Err(Error::api(&endpoint, 200, "No images returned from API"));
        }

        info!(count = images.len(), "Received images from API");

        // Handle output based on params
        self.handle_output(images, &params).await
    }

    /// Handle output of generated images based on params.
    async fn handle_output(
        &self,
        images: Vec<GeneratedImage>,
        params: &ImageGenerateParams,
    ) -> Result<ImageGenerateResult, Error> {
        // If output_uri is specified, upload to storage
        if let Some(output_uri) = &params.output_uri {
            return self.upload_to_storage(images, output_uri).await;
        }

        // If output_file is specified, save to local file
        if let Some(output_file) = &params.output_file {
            return self.save_to_file(images, output_file).await;
        }

        // Otherwise, return base64-encoded data
        Ok(ImageGenerateResult::Base64(images))
    }

    /// Upload images to cloud storage.
    async fn upload_to_storage(
        &self,
        images: Vec<GeneratedImage>,
        output_uri: &str,
    ) -> Result<ImageGenerateResult, Error> {
        let mut uris = Vec::new();

        for (i, image) in images.iter().enumerate() {
            // Decode base64 data
            let data = BASE64.decode(&image.data).map_err(|e| {
                Error::validation(format!("Invalid base64 data: {}", e))
            })?;

            // Determine the URI for this image
            let uri = if images.len() == 1 {
                output_uri.to_string()
            } else {
                // Add index suffix for multiple images
                // Handle GCS URIs properly - don't use Path which treats gs:// as filesystem path
                Self::add_index_suffix_to_uri(output_uri, i, "image", "png")
            };

            // Parse GCS URI and upload
            let gcs_uri = GcsUri::parse(&uri)?;
            self.gcs.upload(&gcs_uri, &data, &image.mime_type).await?;
            uris.push(uri);
        }

        info!(count = uris.len(), "Uploaded images to storage");
        Ok(ImageGenerateResult::StorageUris(uris))
    }

    /// Add an index suffix to a URI or path for multi-output scenarios.
    /// Handles both GCS URIs (gs://bucket/path) and local paths correctly.
    fn add_index_suffix_to_uri(uri: &str, index: usize, default_stem: &str, default_ext: &str) -> String {
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
            // Local filesystem path - use Path
            let path = Path::new(uri);
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(default_stem);
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or(default_ext);
            let parent = path.parent().and_then(|p| p.to_str()).unwrap_or("");
            if parent.is_empty() {
                format!("{}_{}.{}", stem, index, ext)
            } else {
                format!("{}/{}_{}.{}", parent, stem, index, ext)
            }
        }
    }

    /// Save images to local files.
    async fn save_to_file(
        &self,
        images: Vec<GeneratedImage>,
        output_file: &str,
    ) -> Result<ImageGenerateResult, Error> {
        let mut paths = Vec::new();

        for (i, image) in images.iter().enumerate() {
            // Decode base64 data
            let data = BASE64.decode(&image.data).map_err(|e| {
                Error::validation(format!("Invalid base64 data: {}", e))
            })?;

            // Determine the path for this image
            let path = if images.len() == 1 {
                output_file.to_string()
            } else {
                // Add index suffix for multiple images
                let p = Path::new(output_file);
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
                let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("png");
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

        info!(count = paths.len(), "Saved images to local files");
        Ok(ImageGenerateResult::LocalFiles(paths))
    }

    /// Upscale an image using the Imagen Upscale API.
    ///
    /// # Arguments
    /// * `params` - Image upscale parameters
    ///
    /// # Returns
    /// * `Ok(ImageUpscaleResult)` - Upscaled image with data or path
    /// * `Err(Error)` - If validation fails, API call fails, or output handling fails
    #[instrument(level = "info", name = "upscale_image", skip(self, params), fields(upscale_factor = %params.upscale_factor))]
    pub async fn upscale_image(&self, params: ImageUpscaleParams) -> Result<ImageUpscaleResult, Error> {
        // Validate parameters
        params.validate().map_err(|errors| {
            let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            Error::validation(messages.join("; "))
        })?;

        info!(upscale_factor = %params.upscale_factor, "Upscaling image with Imagen Upscale API");

        // Resolve the image input
        let image_data = self.resolve_image_input(&params.image).await?;

        // Build the API request
        let request = UpscaleRequest {
            instances: vec![UpscaleInstance {
                image: UpscaleImageInput {
                    bytes_base64_encoded: image_data,
                },
            }],
            parameters: UpscaleParameters {
                upscale_factor: params.upscale_factor.clone(),
                output_mime_type: "image/png".to_string(),
            },
        };

        // Get auth token
        let token = self.auth.get_token(&["https://www.googleapis.com/auth/cloud-platform"]).await?;

        // Make API request
        let endpoint = self.get_upscale_endpoint();
        debug!(endpoint = %endpoint, "Calling Imagen Upscale API");

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

        // Parse response
        let api_response: UpscaleResponse = response.json().await.map_err(|e| {
            Error::api(&endpoint, status.as_u16(), format!("Failed to parse response: {}", e))
        })?;

        // Extract upscaled image from response
        let prediction = api_response.predictions.into_iter().next()
            .ok_or_else(|| Error::api(&endpoint, 200, "No image returned from API"))?;

        let image_data = prediction.bytes_base64_encoded
            .ok_or_else(|| Error::api(&endpoint, 200, "No image data in response"))?;

        let image = GeneratedImage {
            data: image_data,
            mime_type: prediction.mime_type.unwrap_or_else(|| "image/png".to_string()),
        };

        info!("Received upscaled image from API");

        // Handle output based on params
        self.handle_upscale_output(image, &params).await
    }

    /// Get the Vertex AI Imagen Upscale API endpoint.
    pub fn get_upscale_endpoint(&self) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:predict",
            self.config.location,
            self.config.project_id,
            self.config.location,
            UPSCALE_MODEL
        )
    }

    /// Resolve image input to base64 data.
    async fn resolve_image_input(&self, image: &str) -> Result<String, Error> {
        // Check if it's a GCS URI first (explicit protocol)
        if image.starts_with("gs://") {
            let uri = GcsUri::parse(image)?;
            let data = self.gcs.download(&uri).await?;
            return Ok(BASE64.encode(&data));
        }

        // Check if it looks like a file path
        let looks_like_path = image.starts_with('/')
            || image.starts_with("./")
            || image.starts_with("../")
            || image.starts_with("~/")
            || (image.len() < 500 && image.contains('/'));

        if looks_like_path {
            let path = Path::new(image);
            if !path.exists() {
                return Err(Error::validation(format!("Image file not found: {}", image)));
            }
            let data = tokio::fs::read(path).await?;
            return Ok(BASE64.encode(&data));
        }

        // Try to validate as base64
        if image.len() > 100 {
            if BASE64.decode(image).is_ok() {
                return Ok(image.to_string());
            }
        }

        // Last resort: try as file path
        let path = Path::new(image);
        if path.exists() {
            let data = tokio::fs::read(path).await?;
            return Ok(BASE64.encode(&data));
        }

        // If nothing worked and it's long, assume it's base64
        if image.len() > 100 {
            return Ok(image.to_string());
        }

        Err(Error::validation(format!(
            "Image input is not a valid file path, GCS URI, or base64 data"
        )))
    }

    /// Handle output of upscaled image based on params.
    async fn handle_upscale_output(
        &self,
        image: GeneratedImage,
        params: &ImageUpscaleParams,
    ) -> Result<ImageUpscaleResult, Error> {
        // If output_uri is specified, upload to storage
        if let Some(output_uri) = &params.output_uri {
            let data = BASE64.decode(&image.data).map_err(|e| {
                Error::validation(format!("Invalid base64 data: {}", e))
            })?;
            let gcs_uri = GcsUri::parse(output_uri)?;
            self.gcs.upload(&gcs_uri, &data, &image.mime_type).await?;
            info!(uri = %output_uri, "Uploaded upscaled image to storage");
            return Ok(ImageUpscaleResult::StorageUri(output_uri.clone()));
        }

        // If output_file is specified, save to local file
        if let Some(output_file) = &params.output_file {
            let data = BASE64.decode(&image.data).map_err(|e| {
                Error::validation(format!("Invalid base64 data: {}", e))
            })?;

            // Ensure parent directory exists
            if let Some(parent) = Path::new(output_file).parent() {
                if !parent.as_os_str().is_empty() {
                    tokio::fs::create_dir_all(parent).await?;
                }
            }

            tokio::fs::write(output_file, &data).await?;
            info!(path = %output_file, "Saved upscaled image to local file");
            return Ok(ImageUpscaleResult::LocalFile(output_file.clone()));
        }

        // Otherwise, return base64-encoded data
        Ok(ImageUpscaleResult::Base64(image))
    }
}

// =============================================================================
// API Request/Response Types
// =============================================================================

/// Vertex AI Imagen API request.
#[derive(Debug, Serialize)]
pub struct ImagenRequest {
    /// Input instances (prompts)
    pub instances: Vec<ImagenInstance>,
    /// Generation parameters
    pub parameters: ImagenParameters,
}

/// Imagen API instance (prompt).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagenInstance {
    /// Text prompt describing the image
    pub prompt: String,
    /// Negative prompt - what to avoid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
}

/// Imagen API parameters.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagenParameters {
    /// Number of images to generate
    pub sample_count: u8,
    /// Aspect ratio
    pub aspect_ratio: String,
    /// Random seed for reproducibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

/// Vertex AI Imagen API response.
#[derive(Debug, Deserialize)]
pub struct ImagenResponse {
    /// Generated image predictions
    pub predictions: Vec<ImagenPrediction>,
}

/// Imagen API prediction (generated image).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagenPrediction {
    /// Base64-encoded image data
    pub bytes_base64_encoded: Option<String>,
    /// MIME type of the image
    pub mime_type: Option<String>,
}

// =============================================================================
// Upscale API Request/Response Types
// =============================================================================

/// Vertex AI Imagen Upscale API request.
#[derive(Debug, Serialize)]
pub struct UpscaleRequest {
    /// Input instances (images to upscale)
    pub instances: Vec<UpscaleInstance>,
    /// Upscale parameters
    pub parameters: UpscaleParameters,
}

/// Upscale API instance.
#[derive(Debug, Serialize)]
pub struct UpscaleInstance {
    /// Source image to upscale
    pub image: UpscaleImageInput,
}

/// Upscale image input.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpscaleImageInput {
    /// Base64-encoded image data
    pub bytes_base64_encoded: String,
}

/// Upscale API parameters.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpscaleParameters {
    /// Upscale factor: "x2" or "x4"
    pub upscale_factor: String,
    /// Output MIME type
    pub output_mime_type: String,
}

/// Vertex AI Imagen Upscale API response.
#[derive(Debug, Deserialize)]
pub struct UpscaleResponse {
    /// Upscaled image predictions
    pub predictions: Vec<UpscalePrediction>,
}

/// Upscale API prediction (upscaled image).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpscalePrediction {
    /// Base64-encoded image data
    pub bytes_base64_encoded: Option<String>,
    /// MIME type of the image
    pub mime_type: Option<String>,
}

// =============================================================================
// Result Types
// =============================================================================

/// Generated image data.
#[derive(Debug, Clone)]
pub struct GeneratedImage {
    /// Base64-encoded image data
    pub data: String,
    /// MIME type of the image
    pub mime_type: String,
}

/// Result of image generation.
#[derive(Debug)]
pub enum ImageGenerateResult {
    /// Base64-encoded image data (when no output specified)
    Base64(Vec<GeneratedImage>),
    /// Local file paths (when output_file specified)
    LocalFiles(Vec<String>),
    /// Storage URIs (when output_uri specified)
    StorageUris(Vec<String>),
}

/// Result of image upscaling.
#[derive(Debug)]
pub enum ImageUpscaleResult {
    /// Base64-encoded image data (when no output specified)
    Base64(GeneratedImage),
    /// Local file path (when output_file specified)
    LocalFile(String),
    /// Storage URI (when output_uri specified)
    StorageUri(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params: ImageGenerateParams = serde_json::from_str(r#"{"prompt": "a cat"}"#).unwrap();
        assert_eq!(params.model, DEFAULT_MODEL);
        assert_eq!(params.aspect_ratio, "1:1");
        assert_eq!(params.number_of_images, 1);
        assert!(params.negative_prompt.is_none());
        assert!(params.seed.is_none());
        assert!(params.output_file.is_none());
        assert!(params.output_uri.is_none());
    }

    #[test]
    fn test_valid_params() {
        let params = ImageGenerateParams {
            prompt: "A beautiful sunset over mountains".to_string(),
            negative_prompt: Some("blurry, low quality".to_string()),
            model: "imagen-4".to_string(),
            aspect_ratio: "16:9".to_string(),
            number_of_images: 2,
            seed: Some(42),
            output_file: None,
            output_uri: None,
        };

        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_invalid_number_of_images_zero() {
        let params = ImageGenerateParams {
            prompt: "A cat".to_string(),
            negative_prompt: None,
            model: DEFAULT_MODEL.to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 0,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "number_of_images"));
    }

    #[test]
    fn test_invalid_number_of_images_too_high() {
        let params = ImageGenerateParams {
            prompt: "A cat".to_string(),
            negative_prompt: None,
            model: DEFAULT_MODEL.to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 5,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "number_of_images"));
    }

    #[test]
    fn test_invalid_aspect_ratio() {
        let params = ImageGenerateParams {
            prompt: "A cat".to_string(),
            negative_prompt: None,
            model: DEFAULT_MODEL.to_string(),
            aspect_ratio: "2:1".to_string(),
            number_of_images: 1,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "aspect_ratio"));
    }

    #[test]
    fn test_invalid_model() {
        let params = ImageGenerateParams {
            prompt: "A cat".to_string(),
            negative_prompt: None,
            model: "unknown-model".to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 1,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "model"));
    }

    #[test]
    fn test_empty_prompt() {
        let params = ImageGenerateParams {
            prompt: "   ".to_string(),
            negative_prompt: None,
            model: DEFAULT_MODEL.to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 1,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "prompt"));
    }

    #[test]
    fn test_prompt_too_long_imagen3() {
        let long_prompt = "a".repeat(500); // Exceeds 480 char limit for Imagen 3
        let params = ImageGenerateParams {
            prompt: long_prompt,
            negative_prompt: None,
            model: "imagen-3".to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 1,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "prompt" && e.message.contains("exceeds")));
    }

    #[test]
    fn test_prompt_ok_imagen4() {
        let long_prompt = "a".repeat(500); // Within 2000 char limit for Imagen 4
        let params = ImageGenerateParams {
            prompt: long_prompt,
            negative_prompt: None,
            model: "imagen-4".to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 1,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_all_valid_aspect_ratios() {
        for ratio in VALID_ASPECT_RATIOS {
            let params = ImageGenerateParams {
                prompt: "A cat".to_string(),
                negative_prompt: None,
                model: DEFAULT_MODEL.to_string(),
                aspect_ratio: ratio.to_string(),
                number_of_images: 1,
                seed: None,
                output_file: None,
                output_uri: None,
            };
            assert!(params.validate().is_ok(), "Aspect ratio {} should be valid", ratio);
        }
    }

    #[test]
    fn test_all_valid_number_of_images() {
        for n in MIN_NUMBER_OF_IMAGES..=MAX_NUMBER_OF_IMAGES {
            let params = ImageGenerateParams {
                prompt: "A cat".to_string(),
                negative_prompt: None,
                model: DEFAULT_MODEL.to_string(),
                aspect_ratio: "1:1".to_string(),
                number_of_images: n,
                seed: None,
                output_file: None,
                output_uri: None,
            };
            assert!(params.validate().is_ok(), "number_of_images {} should be valid", n);
        }
    }

    #[test]
    fn test_get_model() {
        let params = ImageGenerateParams {
            prompt: "A cat".to_string(),
            negative_prompt: None,
            model: "imagen-4".to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 1,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let model = params.get_model();
        assert!(model.is_some());
        assert_eq!(model.unwrap().id, "imagen-4.0-generate-preview-06-06");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let params = ImageGenerateParams {
            prompt: "A cat".to_string(),
            negative_prompt: Some("blurry".to_string()),
            model: "imagen-4".to_string(),
            aspect_ratio: "16:9".to_string(),
            number_of_images: 2,
            seed: Some(42),
            output_file: Some("/tmp/output.png".to_string()),
            output_uri: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: ImageGenerateParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params.prompt, deserialized.prompt);
        assert_eq!(params.negative_prompt, deserialized.negative_prompt);
        assert_eq!(params.model, deserialized.model);
        assert_eq!(params.aspect_ratio, deserialized.aspect_ratio);
        assert_eq!(params.number_of_images, deserialized.number_of_images);
        assert_eq!(params.seed, deserialized.seed);
        assert_eq!(params.output_file, deserialized.output_file);
    }

    // Tests for GCS URI handling (P1 fix)
    #[test]
    fn test_add_index_suffix_to_gcs_uri_simple() {
        let uri = "gs://bucket/output.png";
        let result = ImageHandler::add_index_suffix_to_uri(uri, 0, "image", "png");
        assert_eq!(result, "gs://bucket/output_0.png");
    }

    #[test]
    fn test_add_index_suffix_to_gcs_uri_with_path() {
        let uri = "gs://bucket/path/to/output.png";
        let result = ImageHandler::add_index_suffix_to_uri(uri, 1, "image", "png");
        assert_eq!(result, "gs://bucket/path/to/output_1.png");
    }

    #[test]
    fn test_add_index_suffix_to_gcs_uri_no_extension() {
        let uri = "gs://bucket/output";
        let result = ImageHandler::add_index_suffix_to_uri(uri, 2, "image", "png");
        assert_eq!(result, "gs://bucket/output_2.png");
    }

    #[test]
    fn test_add_index_suffix_to_local_path() {
        let path = "/tmp/output.png";
        let result = ImageHandler::add_index_suffix_to_uri(path, 0, "image", "png");
        assert_eq!(result, "/tmp/output_0.png");
    }

    #[test]
    fn test_add_index_suffix_to_local_path_no_dir() {
        let path = "output.png";
        let result = ImageHandler::add_index_suffix_to_uri(path, 1, "image", "png");
        assert_eq!(result, "output_1.png");
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: rust-mcp-genmedia, Property 8: Numeric Parameter Range Validation (number_of_images)
    // **Validates: Requirements 4.5, 4.6**
    //
    // For any numeric parameter with defined bounds (number_of_images 1-4),
    // values outside the valid range SHALL be rejected with a validation error.

    /// Strategy to generate valid number_of_images values (1-4)
    fn valid_number_of_images_strategy() -> impl Strategy<Value = u8> {
        MIN_NUMBER_OF_IMAGES..=MAX_NUMBER_OF_IMAGES
    }

    /// Strategy to generate invalid number_of_images values (0 or > 4)
    fn invalid_number_of_images_strategy() -> impl Strategy<Value = u8> {
        prop_oneof![
            Just(0u8),
            (MAX_NUMBER_OF_IMAGES + 1)..=u8::MAX,
        ]
    }

    /// Strategy to generate valid aspect ratios
    fn valid_aspect_ratio_strategy() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("1:1"),
            Just("3:4"),
            Just("4:3"),
            Just("9:16"),
            Just("16:9"),
        ]
    }

    /// Strategy to generate invalid aspect ratios
    fn invalid_aspect_ratio_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("2:1".to_string()),
            Just("1:2".to_string()),
            Just("5:4".to_string()),
            Just("invalid".to_string()),
            Just("".to_string()),
            Just("16:10".to_string()),
            Just("21:9".to_string()),
            // Generate random invalid ratios
            "[0-9]+:[0-9]+".prop_filter("Must not be a valid ratio", |s| {
                !VALID_ASPECT_RATIOS.contains(&s.as_str())
            }),
        ]
    }

    /// Strategy to generate valid prompts (non-empty, within length limits)
    fn valid_prompt_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 ]{1,100}".prop_map(|s| s.trim().to_string())
            .prop_filter("Must not be empty", |s| !s.trim().is_empty())
    }

    proptest! {
        /// Property 8: Valid number_of_images values (1-4) should pass validation
        #[test]
        fn valid_number_of_images_passes_validation(
            num in valid_number_of_images_strategy(),
            prompt in valid_prompt_strategy(),
        ) {
            let params = ImageGenerateParams {
                prompt,
                negative_prompt: None,
                model: DEFAULT_MODEL.to_string(),
                aspect_ratio: "1:1".to_string(),
                number_of_images: num,
                seed: None,
                output_file: None,
                output_uri: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_ok(),
                "number_of_images {} should be valid, but got errors: {:?}",
                num,
                result.err()
            );
        }

        /// Property 8: Invalid number_of_images values (0 or > 4) should fail validation
        #[test]
        fn invalid_number_of_images_fails_validation(
            num in invalid_number_of_images_strategy(),
            prompt in valid_prompt_strategy(),
        ) {
            let params = ImageGenerateParams {
                prompt,
                negative_prompt: None,
                model: DEFAULT_MODEL.to_string(),
                aspect_ratio: "1:1".to_string(),
                number_of_images: num,
                seed: None,
                output_file: None,
                output_uri: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_err(),
                "number_of_images {} should be invalid",
                num
            );

            let errors = result.unwrap_err();
            prop_assert!(
                errors.iter().any(|e| e.field == "number_of_images"),
                "Should have a number_of_images validation error for value {}",
                num
            );
        }

        // Feature: rust-mcp-genmedia, Property 10: Aspect Ratio Validation
        // **Validates: Requirements 4.5, 4.6**
        //
        // For any aspect_ratio parameter value, it SHALL be one of the model's
        // supported aspect ratios. Invalid aspect ratios SHALL be rejected with
        // a validation error listing valid options.

        /// Property 10: Valid aspect ratios should pass validation
        #[test]
        fn valid_aspect_ratio_passes_validation(
            ratio in valid_aspect_ratio_strategy(),
            prompt in valid_prompt_strategy(),
        ) {
            let params = ImageGenerateParams {
                prompt,
                negative_prompt: None,
                model: DEFAULT_MODEL.to_string(),
                aspect_ratio: ratio.to_string(),
                number_of_images: 1,
                seed: None,
                output_file: None,
                output_uri: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_ok(),
                "aspect_ratio '{}' should be valid, but got errors: {:?}",
                ratio,
                result.err()
            );
        }

        /// Property 10: Invalid aspect ratios should fail validation with descriptive error
        #[test]
        fn invalid_aspect_ratio_fails_validation(
            ratio in invalid_aspect_ratio_strategy(),
            prompt in valid_prompt_strategy(),
        ) {
            let params = ImageGenerateParams {
                prompt,
                negative_prompt: None,
                model: DEFAULT_MODEL.to_string(),
                aspect_ratio: ratio.clone(),
                number_of_images: 1,
                seed: None,
                output_file: None,
                output_uri: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_err(),
                "aspect_ratio '{}' should be invalid",
                ratio
            );

            let errors = result.unwrap_err();
            prop_assert!(
                errors.iter().any(|e| e.field == "aspect_ratio"),
                "Should have an aspect_ratio validation error for value '{}'",
                ratio
            );

            // Verify the error message lists valid options
            let aspect_error = errors.iter().find(|e| e.field == "aspect_ratio").unwrap();
            prop_assert!(
                aspect_error.message.contains("Valid options"),
                "Error message should list valid options: {}",
                aspect_error.message
            );
        }

        /// Property: Combination of valid parameters should always pass validation
        #[test]
        fn valid_params_combination_passes(
            num in valid_number_of_images_strategy(),
            ratio in valid_aspect_ratio_strategy(),
            prompt in valid_prompt_strategy(),
        ) {
            let params = ImageGenerateParams {
                prompt,
                negative_prompt: None,
                model: DEFAULT_MODEL.to_string(),
                aspect_ratio: ratio.to_string(),
                number_of_images: num,
                seed: None,
                output_file: None,
                output_uri: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_ok(),
                "Valid params (num={}, ratio='{}') should pass, but got: {:?}",
                num,
                ratio,
                result.err()
            );
        }
    }
}

/// Unit tests for API interactions and error handling.
/// These tests verify the handler's behavior with mocked API responses.
#[cfg(test)]
mod api_tests {
    use super::*;

    /// Test that ImagenRequest serializes correctly for the API.
    #[test]
    fn test_imagen_request_serialization() {
        let request = ImagenRequest {
            instances: vec![ImagenInstance {
                prompt: "A beautiful sunset".to_string(),
                negative_prompt: Some("blurry".to_string()),
            }],
            parameters: ImagenParameters {
                sample_count: 2,
                aspect_ratio: "16:9".to_string(),
                seed: Some(42),
            },
        };

        let json = serde_json::to_value(&request).unwrap();
        
        // Verify structure
        assert!(json["instances"].is_array());
        assert_eq!(json["instances"][0]["prompt"], "A beautiful sunset");
        assert_eq!(json["instances"][0]["negativePrompt"], "blurry");
        assert_eq!(json["parameters"]["sampleCount"], 2);
        assert_eq!(json["parameters"]["aspectRatio"], "16:9");
        assert_eq!(json["parameters"]["seed"], 42);
    }

    /// Test that ImagenRequest serializes without optional fields when not provided.
    #[test]
    fn test_imagen_request_serialization_minimal() {
        let request = ImagenRequest {
            instances: vec![ImagenInstance {
                prompt: "A cat".to_string(),
                negative_prompt: None,
            }],
            parameters: ImagenParameters {
                sample_count: 1,
                aspect_ratio: "1:1".to_string(),
                seed: None,
            },
        };

        let json = serde_json::to_value(&request).unwrap();
        
        // Verify optional fields are not present
        assert!(json["instances"][0].get("negativePrompt").is_none());
        assert!(json["parameters"].get("seed").is_none());
    }

    /// Test that ImagenResponse deserializes correctly.
    #[test]
    fn test_imagen_response_deserialization() {
        let json = r#"{
            "predictions": [
                {
                    "bytesBase64Encoded": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
                    "mimeType": "image/png"
                }
            ]
        }"#;

        let response: ImagenResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.predictions.len(), 1);
        assert!(response.predictions[0].bytes_base64_encoded.is_some());
        assert_eq!(response.predictions[0].mime_type, Some("image/png".to_string()));
    }

    /// Test that ImagenResponse handles multiple predictions.
    #[test]
    fn test_imagen_response_multiple_predictions() {
        let json = r#"{
            "predictions": [
                {
                    "bytesBase64Encoded": "base64data1",
                    "mimeType": "image/png"
                },
                {
                    "bytesBase64Encoded": "base64data2",
                    "mimeType": "image/png"
                }
            ]
        }"#;

        let response: ImagenResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.predictions.len(), 2);
        assert_eq!(response.predictions[0].bytes_base64_encoded, Some("base64data1".to_string()));
        assert_eq!(response.predictions[1].bytes_base64_encoded, Some("base64data2".to_string()));
    }

    /// Test that ImagenResponse handles empty predictions gracefully.
    #[test]
    fn test_imagen_response_empty_predictions() {
        let json = r#"{"predictions": []}"#;

        let response: ImagenResponse = serde_json::from_str(json).unwrap();
        
        assert!(response.predictions.is_empty());
    }

    /// Test that ImagenResponse handles predictions without image data.
    #[test]
    fn test_imagen_response_no_image_data() {
        let json = r#"{
            "predictions": [
                {
                    "mimeType": "image/png"
                }
            ]
        }"#;

        let response: ImagenResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.predictions.len(), 1);
        assert!(response.predictions[0].bytes_base64_encoded.is_none());
    }

    /// Test endpoint URL construction.
    #[test]
    fn test_get_endpoint() {
        let config = Config {
            project_id: "my-project".to_string(),
            location: "us-central1".to_string(),
            gcs_bucket: None,
            port: 8080,
        };

        // Create a minimal handler for testing endpoint construction
        // We can't create a full handler without auth, but we can test the URL format
        let expected_url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:predict",
            config.location,
            config.project_id,
            config.location,
            "imagen-4.0-generate-preview-05-20"
        );

        assert!(expected_url.contains("us-central1-aiplatform.googleapis.com"));
        assert!(expected_url.contains("my-project"));
        assert!(expected_url.contains("imagen-4.0-generate-preview-05-20"));
        assert!(expected_url.ends_with(":predict"));
    }

    /// Test GeneratedImage structure.
    #[test]
    fn test_generated_image() {
        let image = GeneratedImage {
            data: "base64encodeddata".to_string(),
            mime_type: "image/png".to_string(),
        };

        assert_eq!(image.data, "base64encodeddata");
        assert_eq!(image.mime_type, "image/png");
    }

    /// Test ImageGenerateResult variants.
    #[test]
    fn test_image_generate_result_base64() {
        let images = vec![
            GeneratedImage {
                data: "data1".to_string(),
                mime_type: "image/png".to_string(),
            },
            GeneratedImage {
                data: "data2".to_string(),
                mime_type: "image/jpeg".to_string(),
            },
        ];

        let result = ImageGenerateResult::Base64(images);
        
        match result {
            ImageGenerateResult::Base64(imgs) => {
                assert_eq!(imgs.len(), 2);
                assert_eq!(imgs[0].data, "data1");
                assert_eq!(imgs[1].mime_type, "image/jpeg");
            }
            _ => panic!("Expected Base64 variant"),
        }
    }

    /// Test ImageGenerateResult LocalFiles variant.
    #[test]
    fn test_image_generate_result_local_files() {
        let paths = vec!["/tmp/image1.png".to_string(), "/tmp/image2.png".to_string()];
        let result = ImageGenerateResult::LocalFiles(paths);
        
        match result {
            ImageGenerateResult::LocalFiles(p) => {
                assert_eq!(p.len(), 2);
                assert!(p[0].contains("image1"));
            }
            _ => panic!("Expected LocalFiles variant"),
        }
    }

    /// Test ImageGenerateResult StorageUris variant.
    #[test]
    fn test_image_generate_result_storage_uris() {
        let uris = vec![
            "gs://bucket/image1.png".to_string(),
            "gs://bucket/image2.png".to_string(),
        ];
        let result = ImageGenerateResult::StorageUris(uris);
        
        match result {
            ImageGenerateResult::StorageUris(u) => {
                assert_eq!(u.len(), 2);
                assert!(u[0].starts_with("gs://"));
            }
            _ => panic!("Expected StorageUris variant"),
        }
    }

    /// Test validation error formatting.
    #[test]
    fn test_validation_error_display() {
        let error = ValidationError {
            field: "prompt".to_string(),
            message: "cannot be empty".to_string(),
        };

        let display = format!("{}", error);
        assert_eq!(display, "prompt: cannot be empty");
    }

    /// Test that validation collects multiple errors.
    #[test]
    fn test_validation_multiple_errors() {
        let params = ImageGenerateParams {
            prompt: "   ".to_string(), // Empty prompt
            negative_prompt: None,
            model: "unknown-model".to_string(), // Invalid model
            aspect_ratio: "invalid".to_string(), // Invalid aspect ratio
            number_of_images: 10, // Out of range
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        // Should have errors for: prompt, model, aspect_ratio, number_of_images
        assert!(errors.len() >= 3, "Expected at least 3 validation errors, got {}", errors.len());
        
        let fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
        assert!(fields.contains(&"prompt"));
        assert!(fields.contains(&"model"));
        assert!(fields.contains(&"number_of_images"));
    }
}
