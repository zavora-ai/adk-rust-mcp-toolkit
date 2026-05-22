//! MCP Server for composite media generation.

use crate::{gif, meme, podcast, presentation, short};
use adk_rust_mcp_common::Config;
use rmcp::{
    model::{CallToolResult, Content, ListResourcesResult, ReadResourceResult, ServerCapabilities, ServerInfo},
    ErrorData as McpError, ServerHandler,
};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone)]
pub struct ComposerServer {
    config: Config,
}

impl ComposerServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl ServerHandler for ComposerServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Composite media generation server. Create GIFs, shorts, memes, presentations, and podcasts from text prompts.".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
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

            let tools = vec![
                tool("gif_generate", "Generate an animated GIF from a text prompt. Uses Veo for video generation then converts to optimized GIF.", schema_for!(gif::GifGenerateParams)),
                tool("short_generate", "Generate a vertical short-form video (9:16) with audio and optional caption overlay. Optimized for social media.", schema_for!(short::ShortGenerateParams)),
                tool("meme_generate", "Generate a meme image with top/bottom text overlay from a text prompt.", schema_for!(meme::MemeGenerateParams)),
                tool("presentation_generate", "Generate a narrated video presentation from slides. Each slide gets an AI image, TTS narration, and optional background music.", schema_for!(presentation::PresentationGenerateParams)),
                tool("podcast_generate", "Generate a multi-speaker podcast/dialogue with background music. Each speaker gets a unique voice.", schema_for!(podcast::PodcastGenerateParams)),
            ];

            Ok(ListToolsResult { tools, next_cursor: None, meta: None })
        }
    }

    fn call_tool(
        &self,
        params: rmcp::model::CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            let args = params.arguments.unwrap_or_default();
            let val = serde_json::Value::Object(args);

            let result = match params.name.as_ref() {
                "gif_generate" => {
                    let p: gif::GifGenerateParams = serde_json::from_value(val)
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    gif::generate(&self.config, p).await
                }
                "short_generate" => {
                    let p: short::ShortGenerateParams = serde_json::from_value(val)
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    short::generate(&self.config, p).await
                }
                "meme_generate" => {
                    let p: meme::MemeGenerateParams = serde_json::from_value(val)
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    meme::generate(&self.config, p).await
                }
                "presentation_generate" => {
                    let p: presentation::PresentationGenerateParams = serde_json::from_value(val)
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    presentation::generate(&self.config, p).await
                }
                "podcast_generate" => {
                    let p: podcast::PodcastGenerateParams = serde_json::from_value(val)
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    podcast::generate(&self.config, p).await
                }
                _ => return Err(McpError::invalid_params(format!("Unknown tool: {}", params.name), None)),
            };

            match result {
                Ok(msg) => Ok(CallToolResult::success(vec![Content::text(msg)])),
                Err(e) => Err(McpError::internal_error(e, None)),
            }
        }
    }

    fn list_resources(
        &self, _: Option<rmcp::model::PaginatedRequestParams>,
        _: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async { Ok(ListResourcesResult { resources: vec![], next_cursor: None, meta: None }) }
    }

    fn read_resource(
        &self, params: rmcp::model::ReadResourceRequestParams,
        _: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        async move { Err(McpError::resource_not_found(format!("Unknown: {}", params.uri), None)) }
    }
}

fn tool(name: &'static str, desc: &'static str, schema: schemars::schema::RootSchema) -> rmcp::model::Tool {
    let sv = serde_json::to_value(&schema).unwrap_or_default();
    let is = match sv {
        serde_json::Value::Object(m) => Arc::new(m),
        _ => Arc::new(serde_json::Map::new()),
    };
    rmcp::model::Tool {
        name: Cow::Borrowed(name),
        description: Some(Cow::Borrowed(desc)),
        input_schema: is,
        annotations: None, icons: None, meta: None, output_schema: None, title: None,
    }
}
