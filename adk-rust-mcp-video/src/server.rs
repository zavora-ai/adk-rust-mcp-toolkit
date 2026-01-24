//! MCP Server implementation for the Video server.
//!
//! This module provides the MCP server handler that exposes:
//! - `video_generate` tool for text-to-video generation
//! - `video_from_image` tool for image-to-video generation
//! - `video_extend` tool for video extension
//! - Resources for models and providers

use crate::handler::{VideoT2vParams, VideoI2vParams, VideoExtendParams, VideoGenerateResult, VideoHandler};
use crate::resources;
use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_common::error::Error;
use rmcp::{
    model::{
        CallToolResult, Content, ListResourcesResult, ReadResourceResult,
        ResourceContents, ServerCapabilities, ServerInfo,
    },
    ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// MCP Server for video generation.
#[derive(Clone)]
pub struct VideoServer {
    /// Handler for video generation operations
    handler: Arc<RwLock<Option<VideoHandler>>>,
    /// Server configuration
    config: Config,
}

/// Tool parameters wrapper for video_generate (text-to-video).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VideoGenerateToolParams {
    /// Text prompt describing the video to generate
    pub prompt: String,
    /// Model to use for generation (default: veo-3.0-generate-preview)
    #[serde(default)]
    pub model: Option<String>,
    /// Aspect ratio (16:9, 9:16)
    #[serde(default)]
    pub aspect_ratio: Option<String>,
    /// Duration in seconds (5-8)
    #[serde(default)]
    pub duration_seconds: Option<u8>,
    /// GCS URI for output (required)
    pub output_gcs_uri: String,
    /// Whether to download locally after generation
    #[serde(default)]
    pub download_local: Option<bool>,
    /// Local path for download
    #[serde(default)]
    pub local_path: Option<String>,
    /// Whether to generate audio (Veo 3.x only)
    #[serde(default)]
    pub generate_audio: Option<bool>,
    /// Random seed for reproducibility
    #[serde(default)]
    pub seed: Option<i64>,
}

impl From<VideoGenerateToolParams> for VideoT2vParams {
    fn from(params: VideoGenerateToolParams) -> Self {
        Self {
            prompt: params.prompt,
            model: params.model.unwrap_or_else(|| crate::handler::DEFAULT_MODEL.to_string()),
            aspect_ratio: params.aspect_ratio.unwrap_or_else(|| crate::handler::DEFAULT_ASPECT_RATIO.to_string()),
            duration_seconds: params.duration_seconds.unwrap_or(crate::handler::DEFAULT_DURATION_SECONDS),
            output_gcs_uri: params.output_gcs_uri,
            download_local: params.download_local.unwrap_or(false),
            local_path: params.local_path,
            generate_audio: params.generate_audio,
            seed: params.seed,
        }
    }
}

/// Tool parameters wrapper for video_from_image (image-to-video).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VideoFromImageToolParams {
    /// Source image (base64 data, local path, or GCS URI)
    pub image: String,
    /// Text prompt describing the desired video motion
    pub prompt: String,
    /// Last frame image for interpolation mode (base64 data, local path, or GCS URI).
    /// If provided, generates a video interpolating between `image` and `last_frame_image`.
    #[serde(default)]
    pub last_frame_image: Option<String>,
    /// Model to use for generation (default: veo-3.0-generate-preview)
    #[serde(default)]
    pub model: Option<String>,
    /// Aspect ratio (16:9, 9:16)
    #[serde(default)]
    pub aspect_ratio: Option<String>,
    /// Duration in seconds (5-8)
    #[serde(default)]
    pub duration_seconds: Option<u8>,
    /// GCS URI for output (required)
    pub output_gcs_uri: String,
    /// Whether to download locally after generation
    #[serde(default)]
    pub download_local: Option<bool>,
    /// Local path for download
    #[serde(default)]
    pub local_path: Option<String>,
    /// Random seed for reproducibility
    #[serde(default)]
    pub seed: Option<i64>,
}

impl From<VideoFromImageToolParams> for VideoI2vParams {
    fn from(params: VideoFromImageToolParams) -> Self {
        Self {
            image: params.image,
            prompt: params.prompt,
            last_frame_image: params.last_frame_image,
            model: params.model.unwrap_or_else(|| crate::handler::DEFAULT_MODEL.to_string()),
            aspect_ratio: params.aspect_ratio.unwrap_or_else(|| crate::handler::DEFAULT_ASPECT_RATIO.to_string()),
            duration_seconds: params.duration_seconds.unwrap_or(crate::handler::DEFAULT_DURATION_SECONDS),
            output_gcs_uri: params.output_gcs_uri,
            download_local: params.download_local.unwrap_or(false),
            local_path: params.local_path,
            seed: params.seed,
        }
    }
}

/// Tool parameters wrapper for video_extend (video extension).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VideoExtendToolParams {
    /// GCS URI of the video to extend
    pub video_input: String,
    /// Text prompt describing the desired continuation
    pub prompt: String,
    /// Model to use for generation (default: veo-3.0-generate-preview)
    #[serde(default)]
    pub model: Option<String>,
    /// Duration in seconds (5-8)
    #[serde(default)]
    pub duration_seconds: Option<u8>,
    /// GCS URI for output (required)
    pub output_gcs_uri: String,
    /// Whether to download locally after generation
    #[serde(default)]
    pub download_local: Option<bool>,
    /// Local path for download
    #[serde(default)]
    pub local_path: Option<String>,
    /// Random seed for reproducibility
    #[serde(default)]
    pub seed: Option<i64>,
}

impl From<VideoExtendToolParams> for VideoExtendParams {
    fn from(params: VideoExtendToolParams) -> Self {
        Self {
            video_input: params.video_input,
            prompt: params.prompt,
            model: params.model.unwrap_or_else(|| crate::handler::DEFAULT_MODEL.to_string()),
            duration_seconds: params.duration_seconds.unwrap_or(crate::handler::DEFAULT_DURATION_SECONDS),
            output_gcs_uri: params.output_gcs_uri,
            download_local: params.download_local.unwrap_or(false),
            local_path: params.local_path,
            seed: params.seed,
        }
    }
}

impl VideoServer {
    /// Create a new VideoServer with the given configuration.
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
            *handler = Some(VideoHandler::new(self.config.clone()).await?);
        }
        Ok(())
    }

    /// Generate video from a text prompt.
    pub async fn generate_video(&self, params: VideoGenerateToolParams) -> Result<CallToolResult, McpError> {
        info!(prompt = %params.prompt, "Generating video (text-to-video)");

        // Ensure handler is initialized
        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let gen_params: VideoT2vParams = params.into();
        let result = handler.generate_video_t2v(gen_params).await.map_err(|e| {
            McpError::internal_error(format!("Video generation failed: {}", e), None)
        })?;

        // Convert result to MCP content
        let content = self.format_result(&result);
        Ok(CallToolResult::success(content))
    }

    /// Generate video from an image.
    pub async fn generate_video_from_image(&self, params: VideoFromImageToolParams) -> Result<CallToolResult, McpError> {
        info!(prompt = %params.prompt, "Generating video (image-to-video)");

        // Ensure handler is initialized
        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let gen_params: VideoI2vParams = params.into();
        let result = handler.generate_video_i2v(gen_params).await.map_err(|e| {
            McpError::internal_error(format!("Video generation failed: {}", e), None)
        })?;

        // Convert result to MCP content
        let content = self.format_result(&result);
        Ok(CallToolResult::success(content))
    }

    /// Extend an existing video.
    pub async fn extend_video(&self, params: VideoExtendToolParams) -> Result<CallToolResult, McpError> {
        info!(prompt = %params.prompt, "Extending video");

        // Ensure handler is initialized
        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let extend_params: VideoExtendParams = params.into();
        let result = handler.extend_video(extend_params).await.map_err(|e| {
            McpError::internal_error(format!("Video extension failed: {}", e), None)
        })?;

        // Convert result to MCP content
        let content = self.format_result(&result);
        Ok(CallToolResult::success(content))
    }

    /// Format the video generation result as MCP content.
    fn format_result(&self, result: &VideoGenerateResult) -> Vec<Content> {
        let mut message = format!("Video generated: {}", result.gcs_uri);
        if let Some(local_path) = &result.local_path {
            message.push_str(&format!("\nDownloaded to: {}", local_path));
        }
        vec![Content::text(message)]
    }
}

impl ServerHandler for VideoServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Video generation server using Google Vertex AI Veo API. \
                 Use video_generate for text-to-video, video_from_image for image-to-video, \
                 and video_extend to extend existing videos."
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
        _params: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListToolsResult, McpError>> + Send + '_ {
        async move {
            use rmcp::model::{ListToolsResult, Tool};
            use schemars::schema_for;

            // video_generate tool
            let t2v_schema = schema_for!(VideoGenerateToolParams);
            let t2v_schema_value = serde_json::to_value(&t2v_schema).unwrap_or_default();
            let t2v_input_schema = match t2v_schema_value {
                serde_json::Value::Object(map) => Arc::new(map),
                _ => Arc::new(serde_json::Map::new()),
            };

            // video_from_image tool
            let i2v_schema = schema_for!(VideoFromImageToolParams);
            let i2v_schema_value = serde_json::to_value(&i2v_schema).unwrap_or_default();
            let i2v_input_schema = match i2v_schema_value {
                serde_json::Value::Object(map) => Arc::new(map),
                _ => Arc::new(serde_json::Map::new()),
            };

            // video_extend tool
            let extend_schema = schema_for!(VideoExtendToolParams);
            let extend_schema_value = serde_json::to_value(&extend_schema).unwrap_or_default();
            let extend_input_schema = match extend_schema_value {
                serde_json::Value::Object(map) => Arc::new(map),
                _ => Arc::new(serde_json::Map::new()),
            };

            Ok(ListToolsResult {
                tools: vec![
                    Tool {
                        name: Cow::Borrowed("video_generate"),
                        description: Some(Cow::Borrowed(
                            "Generate video from a text prompt using Google's Veo API. \
                             Requires a GCS URI for output. Returns the GCS URI of the generated video."
                        )),
                        input_schema: t2v_input_schema,
                        annotations: None,
                        icons: None,
                        meta: None,
                        output_schema: None,
                        title: None,
                    },
                    Tool {
                        name: Cow::Borrowed("video_from_image"),
                        description: Some(Cow::Borrowed(
                            "Generate video from an image using Google's Veo API. \
                             Accepts base64 image data, local file path, or GCS URI as input. \
                             Supports interpolation mode: provide both `image` (first frame) and \
                             `last_frame_image` (last frame) to generate a video interpolating between them. \
                             Requires a GCS URI for output. Returns the GCS URI of the generated video."
                        )),
                        input_schema: i2v_input_schema,
                        annotations: None,
                        icons: None,
                        meta: None,
                        output_schema: None,
                        title: None,
                    },
                    Tool {
                        name: Cow::Borrowed("video_extend"),
                        description: Some(Cow::Borrowed(
                            "Extend an existing video using Google's Veo API. \
                             Takes a GCS URI of an existing video and generates additional frames \
                             based on the provided prompt. Requires a GCS URI for output. \
                             Returns the GCS URI of the extended video."
                        )),
                        input_schema: extend_input_schema,
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
        params: rmcp::model::CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            match params.name.as_ref() {
                "video_generate" => {
                    let tool_params: VideoGenerateToolParams = params
                        .arguments
                        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                        .transpose()
                        .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?
                        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;

                    self.generate_video(tool_params).await
                }
                "video_from_image" => {
                    let tool_params: VideoFromImageToolParams = params
                        .arguments
                        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                        .transpose()
                        .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?
                        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;

                    self.generate_video_from_image(tool_params).await
                }
                "video_extend" => {
                    let tool_params: VideoExtendToolParams = params
                        .arguments
                        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                        .transpose()
                        .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?
                        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;

                    self.extend_video(tool_params).await
                }
                _ => Err(McpError::invalid_params(format!("Unknown tool: {}", params.name), None)),
            }
        }
    }

    fn list_resources(
        &self,
        _params: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async move {
            debug!("Listing resources");
            
            let models_resource = rmcp::model::Resource {
                raw: rmcp::model::RawResource {
                    uri: "video://models".to_string(),
                    name: "Available Video Models".to_string(),
                    title: None,
                    description: Some("List of available video generation models".to_string()),
                    mime_type: Some("application/json".to_string()),
                    size: None,
                    icons: None,
                    meta: None,
                },
                annotations: None,
            };

            let providers_resource = rmcp::model::Resource {
                raw: rmcp::model::RawResource {
                    uri: "video://providers".to_string(),
                    name: "Available Providers".to_string(),
                    title: None,
                    description: Some("List of available video generation providers".to_string()),
                    mime_type: Some("application/json".to_string()),
                    size: None,
                    icons: None,
                    meta: None,
                },
                annotations: None,
            };

            Ok(ListResourcesResult {
                resources: vec![models_resource, providers_resource],
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn read_resource(
        &self,
        params: rmcp::model::ReadResourceRequestParam,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        async move {
            let uri = &params.uri;
            debug!(uri = %uri, "Reading resource");

            let content = match uri.as_str() {
                "video://models" => resources::models_resource_json(),
                "video://providers" => resources::providers_resource_json(),
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
        let server = VideoServer::new(test_config());
        let info = server.get_info();
        assert!(info.instructions.is_some());
    }

    #[test]
    fn test_t2v_tool_params_conversion() {
        let tool_params = VideoGenerateToolParams {
            prompt: "A cat walking".to_string(),
            model: Some("veo-3".to_string()),
            aspect_ratio: Some("9:16".to_string()),
            duration_seconds: Some(7),
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: Some(true),
            local_path: Some("/tmp/output.mp4".to_string()),
            generate_audio: Some(true),
            seed: Some(42),
        };

        let gen_params: VideoT2vParams = tool_params.into();
        assert_eq!(gen_params.prompt, "A cat walking");
        assert_eq!(gen_params.model, "veo-3");
        assert_eq!(gen_params.aspect_ratio, "9:16");
        assert_eq!(gen_params.duration_seconds, 7);
        assert_eq!(gen_params.output_gcs_uri, "gs://bucket/output.mp4");
        assert!(gen_params.download_local);
        assert_eq!(gen_params.local_path, Some("/tmp/output.mp4".to_string()));
        assert_eq!(gen_params.generate_audio, Some(true));
        assert_eq!(gen_params.seed, Some(42));
    }

    #[test]
    fn test_t2v_tool_params_defaults() {
        let tool_params = VideoGenerateToolParams {
            prompt: "A cat walking".to_string(),
            model: None,
            aspect_ratio: None,
            duration_seconds: None,
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: None,
            local_path: None,
            generate_audio: None,
            seed: None,
        };

        let gen_params: VideoT2vParams = tool_params.into();
        assert_eq!(gen_params.model, crate::handler::DEFAULT_MODEL);
        assert_eq!(gen_params.aspect_ratio, crate::handler::DEFAULT_ASPECT_RATIO);
        assert_eq!(gen_params.duration_seconds, crate::handler::DEFAULT_DURATION_SECONDS);
        assert!(!gen_params.download_local);
    }

    #[test]
    fn test_i2v_tool_params_conversion() {
        let tool_params = VideoFromImageToolParams {
            image: "base64data".to_string(),
            prompt: "The cat starts walking".to_string(),
            last_frame_image: None,
            model: Some("veo-3".to_string()),
            aspect_ratio: Some("9:16".to_string()),
            duration_seconds: Some(6),
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: Some(true),
            local_path: Some("/tmp/output.mp4".to_string()),
            seed: Some(42),
        };

        let gen_params: VideoI2vParams = tool_params.into();
        assert_eq!(gen_params.image, "base64data");
        assert_eq!(gen_params.prompt, "The cat starts walking");
        assert_eq!(gen_params.model, "veo-3");
        assert_eq!(gen_params.aspect_ratio, "9:16");
        assert_eq!(gen_params.duration_seconds, 6);
    }

    #[test]
    fn test_i2v_tool_params_defaults() {
        let tool_params = VideoFromImageToolParams {
            image: "base64data".to_string(),
            prompt: "The cat starts walking".to_string(),
            last_frame_image: None,
            model: None,
            aspect_ratio: None,
            duration_seconds: None,
            output_gcs_uri: "gs://bucket/output.mp4".to_string(),
            download_local: None,
            local_path: None,
            seed: None,
        };

        let gen_params: VideoI2vParams = tool_params.into();
        assert_eq!(gen_params.model, crate::handler::DEFAULT_MODEL);
        assert_eq!(gen_params.aspect_ratio, crate::handler::DEFAULT_ASPECT_RATIO);
        assert_eq!(gen_params.duration_seconds, crate::handler::DEFAULT_DURATION_SECONDS);
        assert!(!gen_params.download_local);
    }
}
