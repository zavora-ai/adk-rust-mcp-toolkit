//! Multimodal generation handler for the MCP Multimodal server.
//!
//! This module provides the `MultimodalHandler` struct and parameter types for
//! image generation and text-to-speech using Google's Gemini API.

use adk_rust_mcp_common::auth::AuthProvider;
use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_common::error::Error;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, instrument};

/// Default model for multimodal image generation.
pub const DEFAULT_IMAGE_MODEL: &str = "gemini-2.5-flash-image";

/// Default model for multimodal TTS.
pub const DEFAULT_TTS_MODEL: &str = "gemini-2.5-flash-preview-tts";

/// Default voice for multimodal TTS.
pub const DEFAULT_VOICE: &str = "Kore";

/// Available Gemini TTS voices.
pub const AVAILABLE_VOICES: &[&str] = &[
    "Zephyr", "Puck", "Charon", "Kore", "Fenrir", "Leda", "Orus", "Aoede",
];

/// Available TTS styles.
pub const AVAILABLE_STYLES: &[&str] = &[
    "neutral", "cheerful", "sad", "angry", "fearful", "surprised", "calm",
];

/// Supported language codes for Gemini TTS.
pub const SUPPORTED_LANGUAGE_CODES: &[(&str, &str)] = &[
    ("en-US", "English (US)"),
    ("en-GB", "English (UK)"),
    ("es-ES", "Spanish (Spain)"),
    ("es-MX", "Spanish (Mexico)"),
    ("fr-FR", "French (France)"),
    ("de-DE", "German (Germany)"),
    ("it-IT", "Italian (Italy)"),
    ("pt-BR", "Portuguese (Brazil)"),
    ("ja-JP", "Japanese (Japan)"),
    ("ko-KR", "Korean (Korea)"),
    ("zh-CN", "Chinese (Simplified)"),
    ("zh-TW", "Chinese (Traditional)"),
    ("ar-XA", "Arabic"),
    ("hi-IN", "Hindi (India)"),
    ("ru-RU", "Russian (Russia)"),
];

/// Multimodal image generation parameters.
///
/// These parameters control image generation via the Gemini API.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct MultimodalImageParams {
    /// Text prompt describing the image to generate.
    pub prompt: String,

    /// Model to use for generation.
    #[serde(default = "default_image_model")]
    pub model: String,

    /// Output file path for saving the image locally.
    /// If not specified, returns base64-encoded data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_file: Option<String>,
}

fn default_image_model() -> String {
    DEFAULT_IMAGE_MODEL.to_string()
}

/// Multimodal TTS parameters.
///
/// These parameters control text-to-speech synthesis via the Gemini API.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct MultimodalTtsParams {
    /// Text to synthesize into speech.
    pub text: String,

    /// Voice name to use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,

    /// Style/tone for the speech (e.g., "cheerful", "calm", "neutral").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// Model to use for TTS.
    #[serde(default = "default_tts_model")]
    pub model: String,

    /// Output file path for saving the audio locally.
    /// If not specified, returns base64-encoded data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_file: Option<String>,
}

fn default_tts_model() -> String {
    DEFAULT_TTS_MODEL.to_string()
}

/// Validation error details.
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

impl MultimodalImageParams {
    /// Validate the parameters.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

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
}

impl MultimodalTtsParams {
    /// Validate the parameters.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate text is not empty
        if self.text.trim().is_empty() {
            errors.push(ValidationError {
                field: "text".to_string(),
                message: "Text cannot be empty".to_string(),
            });
        }

        // Validate voice if provided
        if let Some(ref voice) = self.voice {
            if !AVAILABLE_VOICES.contains(&voice.as_str()) {
                errors.push(ValidationError {
                    field: "voice".to_string(),
                    message: format!(
                        "Invalid voice '{}'. Available voices: {}",
                        voice,
                        AVAILABLE_VOICES.join(", ")
                    ),
                });
            }
        }

        // Validate style if provided
        if let Some(ref style) = self.style {
            if !AVAILABLE_STYLES.contains(&style.as_str()) {
                errors.push(ValidationError {
                    field: "style".to_string(),
                    message: format!(
                        "Invalid style '{}'. Available styles: {}",
                        style,
                        AVAILABLE_STYLES.join(", ")
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

    /// Get the voice name to use, defaulting if not specified.
    pub fn get_voice(&self) -> &str {
        self.voice.as_deref().unwrap_or(DEFAULT_VOICE)
    }
}

/// Multimodal generation handler.
///
/// Handles image generation and TTS requests using the Gemini API.
pub struct MultimodalHandler {
    /// Application configuration.
    pub config: Config,
    /// HTTP client for API requests.
    pub http: reqwest::Client,
    /// Authentication provider.
    pub auth: AuthProvider,
}

impl MultimodalHandler {
    /// Create a new MultimodalHandler with the given configuration.
    ///
    /// # Errors
    /// Returns an error if auth provider initialization fails.
    #[instrument(level = "debug", name = "multimodal_handler_new", skip_all)]
    pub async fn new(config: Config) -> Result<Self, Error> {
        debug!("Initializing MultimodalHandler");

        let auth = AuthProvider::new().await?;
        let http = reqwest::Client::new();

        Ok(Self { config, http, auth })
    }

    /// Create a new MultimodalHandler with provided dependencies (for testing).
    #[cfg(test)]
    pub fn with_deps(config: Config, http: reqwest::Client, auth: AuthProvider) -> Self {
        Self { config, http, auth }
    }

    /// Get the Gemini API endpoint for image generation.
    pub fn get_image_endpoint(&self, model: &str) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            self.config.location,
            self.config.project_id,
            self.config.location,
            model
        )
    }

    /// Get the Gemini API endpoint for TTS.
    pub fn get_tts_endpoint(&self, model: &str) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            self.config.location,
            self.config.project_id,
            self.config.location,
            model
        )
    }


    /// Generate an image from a text prompt using Gemini.
    ///
    /// # Arguments
    /// * `params` - Image generation parameters
    ///
    /// # Returns
    /// * `Ok(ImageGenerateResult)` - Generated image with data or path
    /// * `Err(Error)` - If validation fails, API call fails, or output handling fails
    #[instrument(level = "info", name = "multimodal_generate_image", skip(self, params))]
    pub async fn generate_image(
        &self,
        params: MultimodalImageParams,
    ) -> Result<ImageGenerateResult, Error> {
        // Validate parameters
        params.validate().map_err(|errors| {
            let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            Error::validation(messages.join("; "))
        })?;

        info!(model = %params.model, "Generating image with Gemini API");

        // Build the API request
        let request = GeminiImageRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart::Text {
                    text: format!("Generate an image of: {}", params.prompt),
                }],
            }],
            generation_config: GeminiGenerationConfig {
                response_modalities: vec!["TEXT".to_string(), "IMAGE".to_string()],
                image_config: Some(GeminiImageConfig {
                    aspect_ratio: "1:1".to_string(),
                }),
                temperature: None,
                max_output_tokens: None,
            },
        };

        // Get auth token
        let token = self
            .auth
            .get_token(&["https://www.googleapis.com/auth/cloud-platform"])
            .await?;

        // Make API request
        let endpoint = self.get_image_endpoint(&params.model);
        debug!(endpoint = %endpoint, "Calling Gemini API for image generation");

        let response = self
            .http
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

        // Get raw response text for debugging
        let response_text = response.text().await.map_err(|e| {
            Error::api(&endpoint, status.as_u16(), format!("Failed to read response: {}", e))
        })?;
        
        debug!(response = %response_text, "Raw Gemini image API response");

        // Parse response
        let api_response: GeminiResponse = serde_json::from_str(&response_text).map_err(|e| {
            Error::api(
                &endpoint,
                status.as_u16(),
                format!("Failed to parse response: {}. Raw: {}", e, &response_text[..response_text.len().min(1000)]),
            )
        })?;

        // Extract image from response
        let image = self.extract_image_from_response(&api_response)?;

        info!("Received image from Gemini API");

        // Handle output based on params
        self.handle_image_output(image, &params).await
    }

    /// Synthesize speech from text using Gemini.
    ///
    /// # Arguments
    /// * `params` - TTS parameters
    ///
    /// # Returns
    /// * `Ok(TtsResult)` - Generated audio with data or path
    /// * `Err(Error)` - If validation fails, API call fails, or output handling fails
    #[instrument(level = "info", name = "multimodal_synthesize_speech", skip(self, params))]
    pub async fn synthesize_speech(&self, params: MultimodalTtsParams) -> Result<TtsResult, Error> {
        // Validate parameters
        params.validate().map_err(|errors| {
            let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            Error::validation(messages.join("; "))
        })?;

        let voice = params.get_voice();
        info!(voice = %voice, model = %params.model, "Synthesizing speech with Gemini API");

        // Build the prompt with style if provided
        let prompt = if let Some(ref style) = params.style {
            format!(
                "Say the following text in a {} tone: {}",
                style, params.text
            )
        } else {
            params.text.clone()
        };

        // Build the API request
        let request = GeminiTtsRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart::Text { text: prompt }],
            }],
            generation_config: GeminiTtsGenerationConfig {
                response_modalities: vec!["AUDIO".to_string()],
                speech_config: GeminiSpeechConfig {
                    voice_config: GeminiVoiceConfig {
                        prebuilt_voice_config: GeminiPrebuiltVoiceConfig {
                            voice_name: voice.to_string(),
                        },
                    },
                },
            },
        };

        // Get auth token
        let token = self
            .auth
            .get_token(&["https://www.googleapis.com/auth/cloud-platform"])
            .await?;

        // Make API request
        let endpoint = self.get_tts_endpoint(&params.model);
        debug!(endpoint = %endpoint, "Calling Gemini API for TTS");

        let response = self
            .http
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

        // Get raw response text for debugging
        let response_text = response.text().await.map_err(|e| {
            Error::api(&endpoint, status.as_u16(), format!("Failed to read response: {}", e))
        })?;
        
        debug!(response = %response_text, "Raw Gemini TTS API response");

        // Parse response
        let api_response: GeminiResponse = serde_json::from_str(&response_text).map_err(|e| {
            Error::api(
                &endpoint,
                status.as_u16(),
                format!("Failed to parse response: {}. Raw: {}", e, &response_text[..response_text.len().min(1000)]),
            )
        })?;

        // Extract audio from response
        let audio = self.extract_audio_from_response(&api_response)?;

        info!("Received audio from Gemini API");

        // Handle output based on params
        self.handle_audio_output(audio, &params).await
    }

    /// List available voices.
    pub fn list_voices(&self) -> Vec<VoiceInfo> {
        AVAILABLE_VOICES
            .iter()
            .map(|&name| VoiceInfo {
                name: name.to_string(),
                description: format!("Gemini TTS voice: {}", name),
            })
            .collect()
    }

    /// List supported language codes.
    pub fn list_language_codes(&self) -> Vec<LanguageCodeInfo> {
        SUPPORTED_LANGUAGE_CODES
            .iter()
            .map(|&(code, name)| LanguageCodeInfo {
                code: code.to_string(),
                name: name.to_string(),
            })
            .collect()
    }

    /// Extract image data from Gemini response.
    fn extract_image_from_response(
        &self,
        response: &GeminiResponse,
    ) -> Result<GeneratedImage, Error> {
        for candidate in &response.candidates {
            if let Some(ref content) = candidate.content {
                for part in &content.parts {
                    if let GeminiResponsePart::InlineData { inline_data } = part {
                        return Ok(GeneratedImage {
                            data: inline_data.data.clone(),
                            mime_type: inline_data.mime_type.clone(),
                        });
                    }
                }
            }
        }

        Err(Error::api(
            "gemini",
            200,
            "No image data found in response".to_string(),
        ))
    }

    /// Extract audio data from Gemini response.
    fn extract_audio_from_response(
        &self,
        response: &GeminiResponse,
    ) -> Result<GeneratedAudio, Error> {
        for candidate in &response.candidates {
            if let Some(ref content) = candidate.content {
                for part in &content.parts {
                    if let GeminiResponsePart::InlineData { inline_data } = part {
                        return Ok(GeneratedAudio {
                            data: inline_data.data.clone(),
                            mime_type: inline_data.mime_type.clone(),
                        });
                    }
                }
            }
        }

        Err(Error::api(
            "gemini",
            200,
            "No audio data found in response".to_string(),
        ))
    }

    /// Handle output of generated image based on params.
    async fn handle_image_output(
        &self,
        image: GeneratedImage,
        params: &MultimodalImageParams,
    ) -> Result<ImageGenerateResult, Error> {
        // If output_file is specified, save to local file
        if let Some(output_file) = &params.output_file {
            return self.save_image_to_file(image, output_file).await;
        }

        // Otherwise, return base64-encoded data
        Ok(ImageGenerateResult::Base64(image))
    }

    /// Handle output of generated audio based on params.
    async fn handle_audio_output(
        &self,
        audio: GeneratedAudio,
        params: &MultimodalTtsParams,
    ) -> Result<TtsResult, Error> {
        // If output_file is specified, save to local file
        if let Some(output_file) = &params.output_file {
            return self.save_audio_to_file(audio, output_file).await;
        }

        // Otherwise, return base64-encoded data
        Ok(TtsResult::Base64(audio))
    }

    /// Save image to local file.
    async fn save_image_to_file(
        &self,
        image: GeneratedImage,
        output_file: &str,
    ) -> Result<ImageGenerateResult, Error> {
        // Decode base64 data
        let data = BASE64
            .decode(&image.data)
            .map_err(|e| Error::validation(format!("Invalid base64 data: {}", e)))?;

        // Ensure parent directory exists
        if let Some(parent) = Path::new(output_file).parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }

        // Write to file
        tokio::fs::write(output_file, &data).await?;

        info!(path = %output_file, "Saved image to local file");
        Ok(ImageGenerateResult::LocalFile(output_file.to_string()))
    }

    /// Save audio to local file.
    async fn save_audio_to_file(
        &self,
        audio: GeneratedAudio,
        output_file: &str,
    ) -> Result<TtsResult, Error> {
        // Decode base64 data
        let data = BASE64
            .decode(&audio.data)
            .map_err(|e| Error::validation(format!("Invalid base64 data: {}", e)))?;

        // Ensure parent directory exists
        if let Some(parent) = Path::new(output_file).parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }

        // Write to file
        tokio::fs::write(output_file, &data).await?;

        info!(path = %output_file, "Saved audio to local file");
        Ok(TtsResult::LocalFile(output_file.to_string()))
    }
}


// =============================================================================
// API Request/Response Types
// =============================================================================

/// Gemini API request for image generation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiImageRequest {
    /// Content parts
    pub contents: Vec<GeminiContent>,
    /// Generation configuration
    pub generation_config: GeminiGenerationConfig,
}

/// Gemini API request for TTS.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiTtsRequest {
    /// Content parts
    pub contents: Vec<GeminiContent>,
    /// Generation configuration
    pub generation_config: GeminiTtsGenerationConfig,
}

/// Gemini content structure.
#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiContent {
    /// Role (user or model)
    pub role: String,
    /// Content parts
    pub parts: Vec<GeminiPart>,
}

/// Gemini content part (request).
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GeminiPart {
    /// Text content
    Text { text: String },
}

/// Gemini generation config for image generation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiGenerationConfig {
    /// Response modalities (TEXT, IMAGE, AUDIO)
    pub response_modalities: Vec<String>,
    /// Image configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_config: Option<GeminiImageConfig>,
    /// Temperature for generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Max output tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
}

/// Gemini image configuration.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiImageConfig {
    /// Aspect ratio for generated images
    pub aspect_ratio: String,
}

/// Gemini generation config for TTS.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiTtsGenerationConfig {
    /// Response modalities (AUDIO)
    pub response_modalities: Vec<String>,
    /// Speech configuration
    pub speech_config: GeminiSpeechConfig,
}

/// Gemini speech configuration.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiSpeechConfig {
    /// Voice configuration
    pub voice_config: GeminiVoiceConfig,
}

/// Gemini voice configuration.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiVoiceConfig {
    /// Prebuilt voice configuration
    pub prebuilt_voice_config: GeminiPrebuiltVoiceConfig,
}

/// Gemini prebuilt voice configuration.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiPrebuiltVoiceConfig {
    /// Voice name
    pub voice_name: String,
}

/// Gemini API response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiResponse {
    /// Response candidates
    #[serde(default)]
    pub candidates: Vec<GeminiCandidate>,
}

/// Gemini response candidate.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiCandidate {
    /// Content
    pub content: Option<GeminiResponseContent>,
}

/// Gemini response content.
#[derive(Debug, Deserialize)]
pub struct GeminiResponseContent {
    /// Content parts
    pub parts: Vec<GeminiResponsePart>,
}

/// Gemini response part.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GeminiResponsePart {
    /// Inline data (image or audio)
    InlineData {
        #[serde(rename = "inlineData")]
        inline_data: GeminiInlineData,
    },
    /// Text content
    Text { text: String },
}

/// Gemini inline data (base64 encoded).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiInlineData {
    /// MIME type
    pub mime_type: String,
    /// Base64-encoded data
    pub data: String,
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

/// Generated audio data.
#[derive(Debug, Clone)]
pub struct GeneratedAudio {
    /// Base64-encoded audio data
    pub data: String,
    /// MIME type of the audio
    pub mime_type: String,
}

/// Result of image generation.
#[derive(Debug)]
pub enum ImageGenerateResult {
    /// Base64-encoded image data (when no output specified)
    Base64(GeneratedImage),
    /// Local file path (when output_file specified)
    LocalFile(String),
}

/// Result of TTS synthesis.
#[derive(Debug)]
pub enum TtsResult {
    /// Base64-encoded audio data (when no output specified)
    Base64(GeneratedAudio),
    /// Local file path (when output_file specified)
    LocalFile(String),
}

/// Voice information.
#[derive(Debug, Clone, Serialize)]
pub struct VoiceInfo {
    /// Voice name
    pub name: String,
    /// Voice description
    pub description: String,
}

/// Language code information.
#[derive(Debug, Clone, Serialize)]
pub struct LanguageCodeInfo {
    /// Language code (e.g., "en-US")
    pub code: String,
    /// Language name (e.g., "English (US)")
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_image_params() {
        let params: MultimodalImageParams =
            serde_json::from_str(r#"{"prompt": "A cat"}"#).unwrap();
        assert_eq!(params.model, DEFAULT_IMAGE_MODEL);
        assert!(params.output_file.is_none());
    }

    #[test]
    fn test_valid_image_params() {
        let params = MultimodalImageParams {
            prompt: "A beautiful sunset".to_string(),
            model: DEFAULT_IMAGE_MODEL.to_string(),
            output_file: None,
        };

        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_empty_prompt_image() {
        let params = MultimodalImageParams {
            prompt: "   ".to_string(),
            model: DEFAULT_IMAGE_MODEL.to_string(),
            output_file: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "prompt"));
    }

    #[test]
    fn test_default_tts_params() {
        let params: MultimodalTtsParams =
            serde_json::from_str(r#"{"text": "Hello world"}"#).unwrap();
        assert_eq!(params.model, DEFAULT_TTS_MODEL);
        assert!(params.voice.is_none());
        assert!(params.style.is_none());
        assert!(params.output_file.is_none());
    }

    #[test]
    fn test_valid_tts_params() {
        let params = MultimodalTtsParams {
            text: "Hello world".to_string(),
            voice: Some("Kore".to_string()),
            style: Some("cheerful".to_string()),
            model: DEFAULT_TTS_MODEL.to_string(),
            output_file: None,
        };

        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_empty_text_tts() {
        let params = MultimodalTtsParams {
            text: "   ".to_string(),
            voice: None,
            style: None,
            model: DEFAULT_TTS_MODEL.to_string(),
            output_file: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "text"));
    }

    #[test]
    fn test_invalid_voice() {
        let params = MultimodalTtsParams {
            text: "Hello".to_string(),
            voice: Some("InvalidVoice".to_string()),
            style: None,
            model: DEFAULT_TTS_MODEL.to_string(),
            output_file: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "voice"));
    }

    #[test]
    fn test_invalid_style() {
        let params = MultimodalTtsParams {
            text: "Hello".to_string(),
            voice: None,
            style: Some("invalid_style".to_string()),
            model: DEFAULT_TTS_MODEL.to_string(),
            output_file: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "style"));
    }

    #[test]
    fn test_get_voice_default() {
        let params = MultimodalTtsParams {
            text: "Hello".to_string(),
            voice: None,
            style: None,
            model: DEFAULT_TTS_MODEL.to_string(),
            output_file: None,
        };

        assert_eq!(params.get_voice(), DEFAULT_VOICE);
    }

    #[test]
    fn test_get_voice_custom() {
        let params = MultimodalTtsParams {
            text: "Hello".to_string(),
            voice: Some("Puck".to_string()),
            style: None,
            model: DEFAULT_TTS_MODEL.to_string(),
            output_file: None,
        };

        assert_eq!(params.get_voice(), "Puck");
    }

    #[test]
    fn test_all_valid_voices() {
        for voice in AVAILABLE_VOICES {
            let params = MultimodalTtsParams {
                text: "Hello".to_string(),
                voice: Some(voice.to_string()),
                style: None,
                model: DEFAULT_TTS_MODEL.to_string(),
                output_file: None,
            };
            assert!(
                params.validate().is_ok(),
                "Voice {} should be valid",
                voice
            );
        }
    }

    #[test]
    fn test_all_valid_styles() {
        for style in AVAILABLE_STYLES {
            let params = MultimodalTtsParams {
                text: "Hello".to_string(),
                voice: None,
                style: Some(style.to_string()),
                model: DEFAULT_TTS_MODEL.to_string(),
                output_file: None,
            };
            assert!(
                params.validate().is_ok(),
                "Style {} should be valid",
                style
            );
        }
    }

    #[test]
    fn test_serialization_roundtrip_image() {
        let params = MultimodalImageParams {
            prompt: "A cat".to_string(),
            model: "custom-model".to_string(),
            output_file: Some("/tmp/output.png".to_string()),
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: MultimodalImageParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params.prompt, deserialized.prompt);
        assert_eq!(params.model, deserialized.model);
        assert_eq!(params.output_file, deserialized.output_file);
    }

    #[test]
    fn test_serialization_roundtrip_tts() {
        let params = MultimodalTtsParams {
            text: "Hello world".to_string(),
            voice: Some("Kore".to_string()),
            style: Some("cheerful".to_string()),
            model: "custom-model".to_string(),
            output_file: Some("/tmp/output.wav".to_string()),
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: MultimodalTtsParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params.text, deserialized.text);
        assert_eq!(params.voice, deserialized.voice);
        assert_eq!(params.style, deserialized.style);
        assert_eq!(params.model, deserialized.model);
        assert_eq!(params.output_file, deserialized.output_file);
    }
}
