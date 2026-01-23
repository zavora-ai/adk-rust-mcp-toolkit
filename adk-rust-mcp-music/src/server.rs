//! MCP Server implementation for the Music server.
//!
//! This module provides the MCP server handler that exposes:
//! - `music_generate` tool for music generation

use crate::handler::{MusicGenerateParams, MusicGenerateResult, MusicHandler};
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
use tracing::{debug, info};

/// MCP Server for music generation.
#[derive(Clone)]
pub struct MusicServer {
    /// Handler for music generation operations
    handler: Arc<RwLock<Option<MusicHandler>>>,
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

        // Ensure handler is initialized
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

        // Convert result to MCP content
        let content = match result {
            MusicGenerateResult::Base64(samples) => {
                samples
                    .into_iter()
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
        _params: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListToolsResult, McpError>> + Send + '_ {
        async move {
            use rmcp::model::{ListToolsResult, Tool};
            use schemars::schema_for;

            let schema = schema_for!(MusicGenerateToolParams);
            let schema_value = serde_json::to_value(&schema).unwrap_or_default();
            
            // Convert to Map
            let input_schema = match schema_value {
                serde_json::Value::Object(map) => Arc::new(map),
                _ => Arc::new(serde_json::Map::new()),
            };

            Ok(ListToolsResult {
                tools: vec![Tool {
                    name: Cow::Borrowed("music_generate"),
                    description: Some(Cow::Borrowed(
                        "Generate music from a text prompt using Google's Lyria API. \
                         Returns base64-encoded WAV data, local file paths, or GCS URIs \
                         depending on output parameters."
                    )),
                    input_schema,
                    annotations: None,
                    icons: None,
                    meta: None,
                    output_schema: None,
                    title: None,
                }],
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
                "music_generate" => {
                    let tool_params: MusicGenerateToolParams = params
                        .arguments
                        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                        .transpose()
                        .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?
                        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;

                    self.generate_music(tool_params).await
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
        params: rmcp::model::ReadResourceRequestParam,
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
