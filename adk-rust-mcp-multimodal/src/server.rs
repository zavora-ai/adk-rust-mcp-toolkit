//! MCP Server implementation for the Multimodal server.
//!
//! This module provides the MCP server handler that exposes:
//! - `multimodal_image_generate` tool for image generation using Gemini
//! - `multimodal_speech_synthesize` tool for TTS using Gemini
//! - `multimodal_list_voices` tool for listing available voices
//! - Resources for language codes

use crate::handler::{
    ImageGenerateResult, MultimodalHandler, MultimodalImageParams, MultimodalTtsParams, TtsResult,
};
use crate::resources;
use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_common::error::Error;
use rmcp::{
    model::{
        CallToolResult, Content, ListResourcesResult, ReadResourceResult, ResourceContents,
        ServerCapabilities, ServerInfo,
    },
    ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// MCP Server for multimodal generation.
#[derive(Clone)]
pub struct MultimodalServer {
    /// Handler for multimodal operations
    handler: Arc<RwLock<Option<MultimodalHandler>>>,
    /// Server configuration
    config: Config,
}

/// Tool parameters wrapper for multimodal_image_generate.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImageGenerateToolParams {
    /// Text prompt describing the image to generate
    pub prompt: String,
    /// Model to use for generation
    #[serde(default)]
    pub model: Option<String>,
    /// Output file path for saving locally
    #[serde(default)]
    pub output_file: Option<String>,
}

impl From<ImageGenerateToolParams> for MultimodalImageParams {
    fn from(params: ImageGenerateToolParams) -> Self {
        Self {
            prompt: params.prompt,
            model: params
                .model
                .unwrap_or_else(|| crate::handler::DEFAULT_IMAGE_MODEL.to_string()),
            output_file: params.output_file,
        }
    }
}

/// Tool parameters wrapper for multimodal_speech_synthesize.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SpeechSynthesizeToolParams {
    /// Text to synthesize into speech
    pub text: String,
    /// Voice name to use
    #[serde(default)]
    pub voice: Option<String>,
    /// Style/tone for the speech (e.g., "cheerful", "calm")
    #[serde(default)]
    pub style: Option<String>,
    /// Model to use for TTS
    #[serde(default)]
    pub model: Option<String>,
    /// Output file path for saving locally
    #[serde(default)]
    pub output_file: Option<String>,
}

impl From<SpeechSynthesizeToolParams> for MultimodalTtsParams {
    fn from(params: SpeechSynthesizeToolParams) -> Self {
        Self {
            text: params.text,
            voice: params.voice,
            style: params.style,
            model: params
                .model
                .unwrap_or_else(|| crate::handler::DEFAULT_TTS_MODEL.to_string()),
            output_file: params.output_file,
        }
    }
}

impl MultimodalServer {
    /// Create a new MultimodalServer with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            handler: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Initialize the handler (called lazily on first use).
    async fn ensure_handler(&self) -> Result<(), Error> {
        let mut handler = self.handler.write().await;
        if handler.is_none() {
            *handler = Some(MultimodalHandler::new(self.config.clone()).await?);
        }
        Ok(())
    }


    /// Generate an image from a text prompt.
    pub async fn generate_image(
        &self,
        params: ImageGenerateToolParams,
    ) -> Result<CallToolResult, McpError> {
        info!(prompt = %params.prompt, "Generating image with Gemini");

        // Ensure handler is initialized
        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard
            .as_ref()
            .ok_or_else(|| McpError::internal_error("Handler not initialized", None))?;

        let gen_params: MultimodalImageParams = params.into();
        let result = handler.generate_image(gen_params).await.map_err(|e| {
            McpError::internal_error(format!("Image generation failed: {}", e), None)
        })?;

        // Convert result to MCP content
        let content = match result {
            ImageGenerateResult::Base64(image) => {
                vec![Content::image(image.data, image.mime_type)]
            }
            ImageGenerateResult::LocalFile(path) => {
                vec![Content::text(format!("Image saved to: {}", path))]
            }
        };

        Ok(CallToolResult::success(content))
    }

    /// Synthesize speech from text.
    pub async fn synthesize_speech(
        &self,
        params: SpeechSynthesizeToolParams,
    ) -> Result<CallToolResult, McpError> {
        info!(text_len = params.text.len(), "Synthesizing speech with Gemini");

        // Ensure handler is initialized
        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard
            .as_ref()
            .ok_or_else(|| McpError::internal_error("Handler not initialized", None))?;

        let tts_params: MultimodalTtsParams = params.into();
        let result = handler.synthesize_speech(tts_params).await.map_err(|e| {
            McpError::internal_error(format!("Speech synthesis failed: {}", e), None)
        })?;

        // Convert result to MCP content
        let content = match result {
            TtsResult::Base64(audio) => {
                vec![Content::text(format!(
                    "data:{};base64,{}",
                    audio.mime_type, audio.data
                ))]
            }
            TtsResult::LocalFile(path) => {
                vec![Content::text(format!("Audio saved to: {}", path))]
            }
        };

        Ok(CallToolResult::success(content))
    }

    /// List available voices.
    pub async fn list_voices(&self) -> Result<CallToolResult, McpError> {
        info!("Listing available Gemini TTS voices");

        // Ensure handler is initialized
        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard
            .as_ref()
            .ok_or_else(|| McpError::internal_error("Handler not initialized", None))?;

        let voices = handler.list_voices();

        // Format voices as JSON
        let voices_json = serde_json::to_string_pretty(&voices).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize voices: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(voices_json)]))
    }
}

impl ServerHandler for MultimodalServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Multimodal generation server using Google Gemini API. \
                 Use multimodal_image_generate to create images from text prompts, \
                 multimodal_speech_synthesize for text-to-speech, \
                 and multimodal_list_voices to see available voices."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            ..Default::default()
        }
    }

    fn list_tools(
        &self,
        _params: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListToolsResult, McpError>> + Send + '_
    {
        async move {
            use rmcp::model::{ListToolsResult, Tool};
            use schemars::schema_for;

            // multimodal_image_generate tool
            let image_schema = schema_for!(ImageGenerateToolParams);
            let image_schema_value = serde_json::to_value(&image_schema).unwrap_or_default();
            let image_input_schema = match image_schema_value {
                serde_json::Value::Object(map) => Arc::new(map),
                _ => Arc::new(serde_json::Map::new()),
            };

            // multimodal_speech_synthesize tool
            let speech_schema = schema_for!(SpeechSynthesizeToolParams);
            let speech_schema_value = serde_json::to_value(&speech_schema).unwrap_or_default();
            let speech_input_schema = match speech_schema_value {
                serde_json::Value::Object(map) => Arc::new(map),
                _ => Arc::new(serde_json::Map::new()),
            };

            // multimodal_list_voices tool (no parameters)
            let empty_schema = Arc::new(serde_json::Map::new());

            Ok(ListToolsResult {
                tools: vec![
                    Tool {
                        name: Cow::Borrowed("multimodal_image_generate"),
                        description: Some(Cow::Borrowed(
                            "Generate images from a text prompt using Google's Gemini API. \
                             Returns base64-encoded image data or saves to a local file.",
                        )),
                        input_schema: image_input_schema,
                        annotations: None,
                        icons: None,
                        meta: None,
                        output_schema: None,
                        title: None,
                    },
                    Tool {
                        name: Cow::Borrowed("multimodal_speech_synthesize"),
                        description: Some(Cow::Borrowed(
                            "Convert text to speech using Google's Gemini API. \
                             Supports multiple voices and style/tone control. \
                             Returns base64-encoded audio or saves to a local file.",
                        )),
                        input_schema: speech_input_schema,
                        annotations: None,
                        icons: None,
                        meta: None,
                        output_schema: None,
                        title: None,
                    },
                    Tool {
                        name: Cow::Borrowed("multimodal_list_voices"),
                        description: Some(Cow::Borrowed(
                            "List available Gemini TTS voices.",
                        )),
                        input_schema: empty_schema,
                        annotations: None,
                        icons: None,
                        meta: None,
                        output_schema: None,
                        title: None,
                    },
                ],
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn call_tool(
        &self,
        params: rmcp::model::CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            match params.name.as_ref() {
                "multimodal_image_generate" => {
                    let tool_params: ImageGenerateToolParams = params
                        .arguments
                        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                        .transpose()
                        .map_err(|e| {
                            McpError::invalid_params(format!("Invalid parameters: {}", e), None)
                        })?
                        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;

                    self.generate_image(tool_params).await
                }
                "multimodal_speech_synthesize" => {
                    let tool_params: SpeechSynthesizeToolParams = params
                        .arguments
                        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                        .transpose()
                        .map_err(|e| {
                            McpError::invalid_params(format!("Invalid parameters: {}", e), None)
                        })?
                        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;

                    self.synthesize_speech(tool_params).await
                }
                "multimodal_list_voices" => self.list_voices().await,
                _ => Err(McpError::invalid_params(
                    format!("Unknown tool: {}", params.name),
                    None,
                )),
            }
        }
    }

    fn list_resources(
        &self,
        _params: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async move {
            debug!("Listing resources");

            let language_codes_resource = rmcp::model::Resource {
                raw: rmcp::model::RawResource {
                    uri: "multimodal://language_codes".to_string(),
                    name: "Supported Language Codes".to_string(),
                    title: None,
                    description: Some("List of supported language codes for Gemini TTS".to_string()),
                    mime_type: Some("application/json".to_string()),
                    size: None,
                    icons: None,
                    meta: None,
                },
                annotations: None,
            };

            let voices_resource = rmcp::model::Resource {
                raw: rmcp::model::RawResource {
                    uri: "multimodal://voices".to_string(),
                    name: "Available Voices".to_string(),
                    title: None,
                    description: Some("List of available Gemini TTS voices".to_string()),
                    mime_type: Some("application/json".to_string()),
                    size: None,
                    icons: None,
                    meta: None,
                },
                annotations: None,
            };

            Ok(ListResourcesResult {
                resources: vec![language_codes_resource, voices_resource],
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn read_resource(
        &self,
        params: rmcp::model::ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        async move {
            let uri = &params.uri;
            debug!(uri = %uri, "Reading resource");

            let content = match uri.as_str() {
                "multimodal://language_codes" => resources::language_codes_resource_json(),
                "multimodal://voices" => resources::voices_resource_json(),
                _ => {
                    return Err(McpError::resource_not_found(
                        format!("Unknown resource: {}", uri),
                        None,
                    ));
                }
            };

            Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(content, uri.clone())],
            })
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            project_id: "test-project".to_string(),
            location: "us-central1".to_string(),
            gcs_bucket: None,
            port: 8080,
        }
    }

    #[test]
    fn test_server_info() {
        let server = MultimodalServer::new(test_config());
        let info = server.get_info();
        assert!(info.instructions.is_some());
    }

    #[test]
    fn test_image_tool_params_conversion() {
        let tool_params = ImageGenerateToolParams {
            prompt: "A cat".to_string(),
            model: Some("custom-model".to_string()),
            output_file: Some("/tmp/output.png".to_string()),
        };

        let gen_params: MultimodalImageParams = tool_params.into();
        assert_eq!(gen_params.prompt, "A cat");
        assert_eq!(gen_params.model, "custom-model");
        assert_eq!(gen_params.output_file, Some("/tmp/output.png".to_string()));
    }

    #[test]
    fn test_image_tool_params_defaults() {
        let tool_params = ImageGenerateToolParams {
            prompt: "A cat".to_string(),
            model: None,
            output_file: None,
        };

        let gen_params: MultimodalImageParams = tool_params.into();
        assert_eq!(gen_params.model, crate::handler::DEFAULT_IMAGE_MODEL);
        assert!(gen_params.output_file.is_none());
    }

    #[test]
    fn test_speech_tool_params_conversion() {
        let tool_params = SpeechSynthesizeToolParams {
            text: "Hello world".to_string(),
            voice: Some("Kore".to_string()),
            style: Some("cheerful".to_string()),
            model: Some("custom-model".to_string()),
            output_file: Some("/tmp/output.wav".to_string()),
        };

        let tts_params: MultimodalTtsParams = tool_params.into();
        assert_eq!(tts_params.text, "Hello world");
        assert_eq!(tts_params.voice, Some("Kore".to_string()));
        assert_eq!(tts_params.style, Some("cheerful".to_string()));
        assert_eq!(tts_params.model, "custom-model");
        assert_eq!(tts_params.output_file, Some("/tmp/output.wav".to_string()));
    }

    #[test]
    fn test_speech_tool_params_defaults() {
        let tool_params = SpeechSynthesizeToolParams {
            text: "Hello".to_string(),
            voice: None,
            style: None,
            model: None,
            output_file: None,
        };

        let tts_params: MultimodalTtsParams = tool_params.into();
        assert_eq!(tts_params.model, crate::handler::DEFAULT_TTS_MODEL);
        assert!(tts_params.voice.is_none());
        assert!(tts_params.style.is_none());
    }
}
