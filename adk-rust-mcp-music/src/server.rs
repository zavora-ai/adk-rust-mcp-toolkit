//! MCP Server implementation for the Music server.
//!
//! This module provides the MCP server handler that exposes:
//! - `music_generate` tool for music generation
//! - `music_realtime_start` tool for starting a real-time music session
//! - `music_realtime_steer` tool for steering an active session
//! - `music_realtime_stop` tool for stopping a session and getting audio

use crate::handler::{MusicGenerateParams, MusicGenerateResult, MusicHandler};
use crate::realtime::{MusicGenConfig, SessionManager, WeightedPrompt};
use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_common::error::Error;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
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
use tracing::{debug, info};

/// MCP Server for music generation.
#[derive(Clone)]
pub struct MusicServer {
    /// Handler for music generation operations
    handler: Arc<RwLock<Option<MusicHandler>>>,
    /// Real-time session manager
    session_manager: Arc<SessionManager>,
    /// Server configuration
    config: Config,
}

/// Tool parameters wrapper for music_generate.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MusicGenerateToolParams {
    /// Text prompt describing the music to generate
    pub prompt: String,
    /// Negative prompt - what to avoid in the generated music
    #[serde(default)]
    pub negative_prompt: Option<String>,
    /// Random seed for reproducibility
    #[serde(default)]
    pub seed: Option<i64>,
    /// Number of samples to generate (1-4)
    #[serde(default)]
    pub sample_count: Option<u8>,
    /// Output file path for saving locally
    #[serde(default)]
    pub output_file: Option<String>,
    /// Output GCS URI (e.g., gs://bucket/path)
    #[serde(default)]
    pub output_gcs_uri: Option<String>,
}

/// Tool parameters for music_realtime_start.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MusicRealtimeStartParams {
    /// Weighted prompts describing the music to generate
    pub prompts: Vec<WeightedPrompt>,
    /// Optional generation config (BPM, scale, density, etc.)
    #[serde(default)]
    pub config: Option<MusicGenConfig>,
}

/// Tool parameters for music_realtime_steer.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MusicRealtimeSteerParams {
    /// Session ID returned by music_realtime_start
    pub session_id: String,
    /// New weighted prompts (replaces current prompts)
    #[serde(default)]
    pub prompts: Option<Vec<WeightedPrompt>>,
    /// Updated generation config
    #[serde(default)]
    pub config: Option<MusicGenConfig>,
    /// Playback action: "pause", "resume", or "reset_context"
    #[serde(default)]
    pub action: Option<String>,
}

/// Tool parameters for music_realtime_stop.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MusicRealtimeStopParams {
    /// Session ID returned by music_realtime_start
    pub session_id: String,
    /// Output file path to save the generated audio (WAV, 48kHz stereo)
    #[serde(default)]
    pub output_file: Option<String>,
}

impl From<MusicGenerateToolParams> for MusicGenerateParams {
    fn from(params: MusicGenerateToolParams) -> Self {
        Self {
            prompt: params.prompt,
            negative_prompt: params.negative_prompt,
            seed: params.seed,
            sample_count: params.sample_count.unwrap_or(1),
            output_file: params.output_file,
            output_gcs_uri: params.output_gcs_uri,
        }
    }
}

impl MusicServer {
    /// Create a new MusicServer with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            handler: Arc::new(RwLock::new(None)),
            session_manager: Arc::new(SessionManager::new()),
            config,
        }
    }

    /// Initialize the handler (called lazily on first use).
    async fn ensure_handler(&self) -> Result<(), Error> {
        let mut handler = self.handler.write().await;
        if handler.is_none() {
            *handler = Some(MusicHandler::new(self.config.clone()).await?);
        }
        Ok(())
    }

    /// Generate music from a text prompt.
    pub async fn generate_music(&self, params: MusicGenerateToolParams) -> Result<CallToolResult, McpError> {
        info!(prompt = %params.prompt, "Generating music");

        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let gen_params: MusicGenerateParams = params.into();
        let result = handler.generate_music(gen_params).await.map_err(|e| {
            McpError::internal_error(format!("Music generation failed: {}", e), None)
        })?;

        let content = match result {
            MusicGenerateResult::Base64(samples) => {
                samples.into_iter()
                    .map(|s| Content::text(format!("data:{};base64,{}", s.mime_type, s.data)))
                    .collect()
            }
            MusicGenerateResult::LocalFiles(paths) => {
                vec![Content::text(format!("Audio saved to: {}", paths.join(", ")))]
            }
            MusicGenerateResult::GcsUris(uris) => {
                vec![Content::text(format!("Audio uploaded to: {}", uris.join(", ")))]
            }
        };

        Ok(CallToolResult::success(content))
    }

    /// Start a real-time music generation session.
    pub async fn realtime_start(&self, params: MusicRealtimeStartParams) -> Result<CallToolResult, McpError> {
        let api_key = self.config.gemini_api_key.as_deref()
            .ok_or_else(|| McpError::internal_error("GEMINI_API_KEY required for realtime music", None))?;

        info!(prompts = params.prompts.len(), "Starting realtime music session");

        let session_id = self.session_manager
            .create_session(api_key, params.prompts, params.config)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to start session: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Realtime music session started. Session ID: {}. Audio is streaming (48kHz stereo PCM). Use music_realtime_steer to change prompts/config, or music_realtime_stop to end and save.",
            session_id
        ))]))
    }

    /// Steer an active real-time session.
    pub async fn realtime_steer(&self, params: MusicRealtimeSteerParams) -> Result<CallToolResult, McpError> {
        let session = self.session_manager.get_session(&params.session_id).await
            .ok_or_else(|| McpError::invalid_params(format!("Session not found: {}", params.session_id), None))?;

        if let Some(prompts) = &params.prompts {
            session.send_prompts(prompts).await
                .map_err(|e| McpError::internal_error(format!("Failed to send prompts: {}", e), None))?;
        }

        if let Some(config) = &params.config {
            session.send_config(config).await
                .map_err(|e| McpError::internal_error(format!("Failed to send config: {}", e), None))?;
        }

        if let Some(action) = &params.action {
            match action.as_str() {
                "pause" => session.send_pause().await
                    .map_err(|e| McpError::internal_error(e, None))?,
                "resume" => session.send_play().await
                    .map_err(|e| McpError::internal_error(e, None))?,
                "reset_context" => {
                    let msg = serde_json::json!({"playback_control": "RESET_CONTEXT"});
                    session.send_json_public(&msg).await
                        .map_err(|e| McpError::internal_error(e, None))?;
                }
                _ => return Err(McpError::invalid_params(format!("Unknown action: {}", action), None)),
            }
        }

        let buf_size = session.buffer_size().await;
        let duration_secs = buf_size as f64 / (48000.0 * 2.0 * 2.0); // 48kHz, stereo, 16-bit

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Session {} updated. Buffered audio: {:.1}s ({} bytes)",
            params.session_id, duration_secs, buf_size
        ))]))
    }

    /// Stop a real-time session and return the audio.
    pub async fn realtime_stop(&self, params: MusicRealtimeStopParams) -> Result<CallToolResult, McpError> {
        let session = self.session_manager.remove_session(&params.session_id).await
            .ok_or_else(|| McpError::invalid_params(format!("Session not found: {}", params.session_id), None))?;

        let pcm_data = session.stop().await
            .map_err(|e| McpError::internal_error(format!("Failed to stop session: {}", e), None))?;

        let duration_secs = pcm_data.len() as f64 / (48000.0 * 2.0 * 2.0);

        if let Some(output_file) = &params.output_file {
            // Write WAV file
            let wav_data = pcm_to_wav(&pcm_data, 48000, 2, 16);
            tokio::fs::write(output_file, &wav_data).await
                .map_err(|e| McpError::internal_error(format!("Failed to write file: {}", e), None))?;

            Ok(CallToolResult::success(vec![Content::text(format!(
                "Session stopped. Audio saved to: {} ({:.1}s, 48kHz stereo)",
                output_file, duration_secs
            ))]))
        } else {
            // Return base64-encoded WAV
            let wav_data = pcm_to_wav(&pcm_data, 48000, 2, 16);
            let b64 = BASE64.encode(&wav_data);
            Ok(CallToolResult::success(vec![Content::text(format!(
                "data:audio/wav;base64,{}", b64
            ))]))
        }
    }
}

/// Convert raw PCM data to WAV format.
fn pcm_to_wav(pcm: &[u8], sample_rate: u32, channels: u16, bits_per_sample: u16) -> Vec<u8> {
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    let block_align = channels * bits_per_sample / 8;
    let data_size = pcm.len() as u32;
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + pcm.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    wav.extend_from_slice(pcm);
    wav
}

impl ServerHandler for MusicServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Music generation server using Google Vertex AI Lyria API. \
                 Use the music_generate tool to create music from text prompts."
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
        _params: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListToolsResult, McpError>> + Send + '_ {
        async move {
            use rmcp::model::{ListToolsResult, Tool};
            use schemars::schema_for;

            let gen_schema = schema_for!(MusicGenerateToolParams);
            let gen_sv = serde_json::to_value(&gen_schema).unwrap_or_default();
            let gen_is = match gen_sv { serde_json::Value::Object(m) => Arc::new(m), _ => Arc::new(serde_json::Map::new()) };

            let start_schema = schema_for!(MusicRealtimeStartParams);
            let start_sv = serde_json::to_value(&start_schema).unwrap_or_default();
            let start_is = match start_sv { serde_json::Value::Object(m) => Arc::new(m), _ => Arc::new(serde_json::Map::new()) };

            let steer_schema = schema_for!(MusicRealtimeSteerParams);
            let steer_sv = serde_json::to_value(&steer_schema).unwrap_or_default();
            let steer_is = match steer_sv { serde_json::Value::Object(m) => Arc::new(m), _ => Arc::new(serde_json::Map::new()) };

            let stop_schema = schema_for!(MusicRealtimeStopParams);
            let stop_sv = serde_json::to_value(&stop_schema).unwrap_or_default();
            let stop_is = match stop_sv { serde_json::Value::Object(m) => Arc::new(m), _ => Arc::new(serde_json::Map::new()) };

            Ok(ListToolsResult {
                tools: vec![
                    Tool {
                        name: Cow::Borrowed("music_generate"),
                        description: Some(Cow::Borrowed(
                            "Generate music from a text prompt using Google's Lyria API. Returns base64-encoded audio data, local file paths, or GCS URIs depending on output parameters."
                        )),
                        input_schema: gen_is,
                        annotations: None, icons: None, meta: None, output_schema: None, title: None,
                    },
                    Tool {
                        name: Cow::Borrowed("music_realtime_start"),
                        description: Some(Cow::Borrowed(
                            "Start a real-time music generation session using Lyria RealTime. Returns a session ID. Audio streams continuously at 48kHz stereo. Use music_realtime_steer to change prompts/config, and music_realtime_stop to end and save the audio."
                        )),
                        input_schema: start_is,
                        annotations: None, icons: None, meta: None, output_schema: None, title: None,
                    },
                    Tool {
                        name: Cow::Borrowed("music_realtime_steer"),
                        description: Some(Cow::Borrowed(
                            "Steer an active Lyria RealTime session. Update prompts, config (BPM, scale, density, brightness), or control playback (pause/resume/reset_context)."
                        )),
                        input_schema: steer_is,
                        annotations: None, icons: None, meta: None, output_schema: None, title: None,
                    },
                    Tool {
                        name: Cow::Borrowed("music_realtime_stop"),
                        description: Some(Cow::Borrowed(
                            "Stop a Lyria RealTime session and save the accumulated audio as a WAV file (48kHz stereo 16-bit PCM)."
                        )),
                        input_schema: stop_is,
                        annotations: None, icons: None, meta: None, output_schema: None, title: None,
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
                "music_generate" => {
                    let tool_params: MusicGenerateToolParams = params
                        .arguments
                        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                        .transpose()
                        .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?
                        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;
                    self.generate_music(tool_params).await
                }
                "music_realtime_start" => {
                    let tool_params: MusicRealtimeStartParams = params
                        .arguments
                        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                        .transpose()
                        .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?
                        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;
                    self.realtime_start(tool_params).await
                }
                "music_realtime_steer" => {
                    let tool_params: MusicRealtimeSteerParams = params
                        .arguments
                        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                        .transpose()
                        .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?
                        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;
                    self.realtime_steer(tool_params).await
                }
                "music_realtime_stop" => {
                    let tool_params: MusicRealtimeStopParams = params
                        .arguments
                        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                        .transpose()
                        .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?
                        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;
                    self.realtime_stop(tool_params).await
                }
                _ => Err(McpError::invalid_params(format!("Unknown tool: {}", params.name), None)),
            }
        }
    }

    fn list_resources(
        &self,
        _params: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async move {
            debug!("Listing resources (none available for music server)");
            
            Ok(ListResourcesResult {
                resources: vec![],
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

            Err(McpError::resource_not_found(
                format!("Unknown resource: {}", uri),
                None,
            ))
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
        ..Default::default()
        }
    }

    #[test]
    fn test_server_info() {
        let server = MusicServer::new(test_config());
        let info = server.get_info();
        assert!(info.instructions.is_some());
    }

    #[test]
    fn test_tool_params_conversion() {
        let tool_params = MusicGenerateToolParams {
            prompt: "A jazz tune".to_string(),
            negative_prompt: Some("vocals".to_string()),
            seed: Some(42),
            sample_count: Some(2),
            output_file: None,
            output_gcs_uri: None,
        };

        let gen_params: MusicGenerateParams = tool_params.into();
        assert_eq!(gen_params.prompt, "A jazz tune");
        assert_eq!(gen_params.negative_prompt, Some("vocals".to_string()));
        assert_eq!(gen_params.seed, Some(42));
        assert_eq!(gen_params.sample_count, 2);
    }

    #[test]
    fn test_tool_params_defaults() {
        let tool_params = MusicGenerateToolParams {
            prompt: "A song".to_string(),
            negative_prompt: None,
            seed: None,
            sample_count: None,
            output_file: None,
            output_gcs_uri: None,
        };

        let gen_params: MusicGenerateParams = tool_params.into();
        assert_eq!(gen_params.sample_count, 1);
    }
}
