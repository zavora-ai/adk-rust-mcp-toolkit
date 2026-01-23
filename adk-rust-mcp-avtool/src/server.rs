//! MCP Server implementation for the AVTool server.
//!
//! This module provides the MCP server handler that exposes FFmpeg-based
//! audio/video processing tools.

use crate::handler::{
    AVToolHandler, AdjustVolumeParams, CombineAvParams, ConcatenateParams,
    ConvertAudioParams, GetMediaInfoParams, LayerAudioParams,
    OverlayImageParams, VideoToGifParams,
};
use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_common::error::Error;
use rmcp::{
    model::{
        CallToolResult, Content, ListResourcesResult, ReadResourceResult,
        ServerCapabilities, ServerInfo,
    },
    ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// MCP Server for audio/video processing.
#[derive(Clone)]
pub struct AVToolServer {
    /// Handler for FFmpeg operations
    handler: Arc<RwLock<Option<AVToolHandler>>>,
    /// Server configuration
    config: Config,
}

impl AVToolServer {
    /// Create a new AVToolServer with the given configuration.
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
            *handler = Some(AVToolHandler::new(self.config.clone()).await?);
        }
        Ok(())
    }

    /// Get media file information.
    pub async fn get_media_info(&self, params: GetMediaInfoParams) -> Result<CallToolResult, McpError> {
        info!(input = %params.input, "Getting media info");

        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let info = handler.get_media_info(params).await.map_err(|e| {
            McpError::internal_error(format!("Failed to get media info: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&info).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Convert WAV to MP3.
    pub async fn convert_wav_to_mp3(&self, params: ConvertAudioParams) -> Result<CallToolResult, McpError> {
        info!(input = %params.input, output = %params.output, "Converting WAV to MP3");

        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let output = handler.convert_wav_to_mp3(params).await.map_err(|e| {
            McpError::internal_error(format!("Conversion failed: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!("Converted to: {}", output))]))
    }

    /// Convert video to GIF.
    pub async fn video_to_gif(&self, params: VideoToGifParams) -> Result<CallToolResult, McpError> {
        info!(input = %params.input, output = %params.output, "Converting video to GIF");

        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let output = handler.video_to_gif(params).await.map_err(|e| {
            McpError::internal_error(format!("Conversion failed: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!("Created GIF: {}", output))]))
    }

    /// Combine audio and video.
    pub async fn combine_audio_video(&self, params: CombineAvParams) -> Result<CallToolResult, McpError> {
        info!(video = %params.video_input, audio = %params.audio_input, "Combining audio and video");

        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let output = handler.combine_audio_video(params).await.map_err(|e| {
            McpError::internal_error(format!("Combine failed: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!("Combined to: {}", output))]))
    }

    /// Overlay image on video.
    pub async fn overlay_image(&self, params: OverlayImageParams) -> Result<CallToolResult, McpError> {
        info!(video = %params.video_input, image = %params.image_input, "Overlaying image on video");

        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let output = handler.overlay_image(params).await.map_err(|e| {
            McpError::internal_error(format!("Overlay failed: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!("Created: {}", output))]))
    }

    /// Concatenate media files.
    pub async fn concatenate(&self, params: ConcatenateParams) -> Result<CallToolResult, McpError> {
        info!(count = params.inputs.len(), output = %params.output, "Concatenating media files");

        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let output = handler.concatenate(params).await.map_err(|e| {
            McpError::internal_error(format!("Concatenation failed: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!("Concatenated to: {}", output))]))
    }

    /// Adjust audio volume.
    pub async fn adjust_volume(&self, params: AdjustVolumeParams) -> Result<CallToolResult, McpError> {
        info!(input = %params.input, volume = %params.volume, "Adjusting audio volume");

        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let output = handler.adjust_volume(params).await.map_err(|e| {
            McpError::internal_error(format!("Volume adjustment failed: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!("Adjusted volume: {}", output))]))
    }

    /// Layer multiple audio files.
    pub async fn layer_audio(&self, params: LayerAudioParams) -> Result<CallToolResult, McpError> {
        info!(layers = params.inputs.len(), output = %params.output, "Layering audio files");

        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let output = handler.layer_audio(params).await.map_err(|e| {
            McpError::internal_error(format!("Audio layering failed: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!("Layered audio: {}", output))]))
    }
}

impl ServerHandler for AVToolServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Audio/video processing server using FFmpeg. \
                 Provides tools for media conversion, combining, and manipulation."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
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
            use rmcp::model::ListToolsResult;

            let tools = vec![
                create_tool::<GetMediaInfoParams>(
                    "ffmpeg_get_media_info",
                    "Get information about a media file (duration, format, streams, codecs).",
                ),
                create_tool::<ConvertAudioParams>(
                    "ffmpeg_convert_audio_wav_to_mp3",
                    "Convert a WAV audio file to MP3 format with configurable bitrate.",
                ),
                create_tool::<VideoToGifParams>(
                    "ffmpeg_video_to_gif",
                    "Convert a video file to animated GIF with configurable FPS, width, and duration.",
                ),
                create_tool::<CombineAvParams>(
                    "ffmpeg_combine_audio_and_video",
                    "Combine separate audio and video files into a single file.",
                ),
                create_tool::<OverlayImageParams>(
                    "ffmpeg_overlay_image_on_video",
                    "Overlay an image on a video at a specified position with optional timing.",
                ),
                create_tool::<ConcatenateParams>(
                    "ffmpeg_concatenate_media_files",
                    "Concatenate multiple media files into a single file.",
                ),
                create_tool::<AdjustVolumeParams>(
                    "ffmpeg_adjust_volume",
                    "Adjust the volume of an audio file using multiplier or dB notation.",
                ),
                create_tool::<LayerAudioParams>(
                    "ffmpeg_layer_audio_files",
                    "Layer/mix multiple audio files with optional offset and volume control.",
                ),
            ];

            Ok(ListToolsResult {
                tools,
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
                "ffmpeg_get_media_info" => {
                    let tool_params: GetMediaInfoParams = parse_params(params.arguments)?;
                    self.get_media_info(tool_params).await
                }
                "ffmpeg_convert_audio_wav_to_mp3" => {
                    let tool_params: ConvertAudioParams = parse_params(params.arguments)?;
                    self.convert_wav_to_mp3(tool_params).await
                }
                "ffmpeg_video_to_gif" => {
                    let tool_params: VideoToGifParams = parse_params(params.arguments)?;
                    self.video_to_gif(tool_params).await
                }
                "ffmpeg_combine_audio_and_video" => {
                    let tool_params: CombineAvParams = parse_params(params.arguments)?;
                    self.combine_audio_video(tool_params).await
                }
                "ffmpeg_overlay_image_on_video" => {
                    let tool_params: OverlayImageParams = parse_params(params.arguments)?;
                    self.overlay_image(tool_params).await
                }
                "ffmpeg_concatenate_media_files" => {
                    let tool_params: ConcatenateParams = parse_params(params.arguments)?;
                    self.concatenate(tool_params).await
                }
                "ffmpeg_adjust_volume" => {
                    let tool_params: AdjustVolumeParams = parse_params(params.arguments)?;
                    self.adjust_volume(tool_params).await
                }
                "ffmpeg_layer_audio_files" => {
                    let tool_params: LayerAudioParams = parse_params(params.arguments)?;
                    self.layer_audio(tool_params).await
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
            // AVTool server doesn't expose any resources
            Ok(ListResourcesResult {
                resources: vec![],
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
            Err(McpError::resource_not_found(
                format!("Unknown resource: {}", params.uri),
                None,
            ))
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Create a tool definition from a parameter type.
fn create_tool<T: JsonSchema>(name: &'static str, description: &'static str) -> rmcp::model::Tool {
    use schemars::schema_for;

    let schema = schema_for!(T);
    let schema_value = serde_json::to_value(&schema).unwrap_or_default();
    
    let input_schema = match schema_value {
        serde_json::Value::Object(map) => Arc::new(map),
        _ => Arc::new(serde_json::Map::new()),
    };

    rmcp::model::Tool {
        name: Cow::Borrowed(name),
        description: Some(Cow::Borrowed(description)),
        input_schema,
        annotations: None,
        icons: None,
        meta: None,
        output_schema: None,
        title: None,
    }
}

/// Parse tool parameters from JSON arguments.
fn parse_params<T: for<'de> Deserialize<'de>>(
    arguments: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<T, McpError> {
    arguments
        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
        .transpose()
        .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?
        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))
}

// =============================================================================
// Tests
// =============================================================================

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
        let server = AVToolServer::new(test_config());
        let info = server.get_info();
        assert!(info.instructions.is_some());
        assert!(info.instructions.unwrap().contains("FFmpeg"));
    }

    #[test]
    fn test_create_tool() {
        let tool = create_tool::<GetMediaInfoParams>(
            "ffmpeg_get_media_info",
            "Get media info",
        );
        assert_eq!(tool.name.as_ref(), "ffmpeg_get_media_info");
        assert!(tool.description.is_some());
    }

    #[test]
    fn test_parse_params_valid() {
        let mut args = serde_json::Map::new();
        args.insert("input".to_string(), serde_json::Value::String("test.mp4".to_string()));
        
        let result: Result<GetMediaInfoParams, _> = parse_params(Some(args));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().input, "test.mp4");
    }

    #[test]
    fn test_parse_params_missing() {
        let result: Result<GetMediaInfoParams, _> = parse_params(None);
        assert!(result.is_err());
    }
}
