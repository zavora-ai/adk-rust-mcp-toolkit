//! MCP Server for artistic image creation.

use crate::{create, sketch_to_art, style_transfer, variations};
use adk_rust_mcp_common::Config;
use rmcp::{
    model::{CallToolResult, Content, ListResourcesResult, ReadResourceResult, ServerCapabilities, ServerInfo},
    ErrorData as McpError, ServerHandler,
};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone)]
pub struct ArtistServer {
    config: Config,
}

impl ArtistServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl ServerHandler for ArtistServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Artistic image creation server. Create art, transfer styles, convert sketches, and generate variations.".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    fn list_tools(
        &self, _: Option<rmcp::model::PaginatedRequestParams>,
        _: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListToolsResult, McpError>> + Send + '_ {
        async move {
            use rmcp::model::ListToolsResult;
            use schemars::schema_for;
            Ok(ListToolsResult {
                tools: vec![
                    tool("artist_create", "Create art from text in a specific style", schema_for!(create::ArtistCreateParams)),
                    tool("artist_style_transfer", "Apply an art style from one image to another", schema_for!(style_transfer::StyleTransferParams)),
                    tool("artist_sketch_to_art", "Turn a rough sketch into finished artwork", schema_for!(sketch_to_art::SketchToArtParams)),
                    tool("artist_variations", "Generate style variations of an existing image", schema_for!(variations::VariationsParams)),
                ],
                next_cursor: None, meta: None,
            })
        }
    }

    fn call_tool(
        &self, params: rmcp::model::CallToolRequestParams,
        _: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            let val = serde_json::Value::Object(params.arguments.unwrap_or_default());
            let result = match params.name.as_ref() {
                "artist_create" => {
                    let p: create::ArtistCreateParams = serde_json::from_value(val).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    create::generate(&self.config, p).await
                }
                "artist_style_transfer" => {
                    let p: style_transfer::StyleTransferParams = serde_json::from_value(val).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    style_transfer::generate(&self.config, p).await
                }
                "artist_sketch_to_art" => {
                    let p: sketch_to_art::SketchToArtParams = serde_json::from_value(val).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    sketch_to_art::generate(&self.config, p).await
                }
                "artist_variations" => {
                    let p: variations::VariationsParams = serde_json::from_value(val).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    variations::generate(&self.config, p).await
                }
                _ => return Err(McpError::invalid_params(format!("Unknown tool: {}", params.name), None)),
            };
            match result {
                Ok(msg) => Ok(CallToolResult::success(vec![Content::text(msg)])),
                Err(e) => Err(McpError::internal_error(e, None)),
            }
        }
    }

    fn list_resources(&self, _: Option<rmcp::model::PaginatedRequestParams>, _: rmcp::service::RequestContext<rmcp::service::RoleServer>) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async { Ok(ListResourcesResult { resources: vec![], next_cursor: None, meta: None }) }
    }

    fn read_resource(&self, params: rmcp::model::ReadResourceRequestParams, _: rmcp::service::RequestContext<rmcp::service::RoleServer>) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        async move { Err(McpError::resource_not_found(format!("Unknown: {}", params.uri), None)) }
    }
}

fn tool(name: &'static str, desc: &'static str, schema: schemars::schema::RootSchema) -> rmcp::model::Tool {
    let sv = serde_json::to_value(&schema).unwrap_or_default();
    let is = match sv { serde_json::Value::Object(m) => Arc::new(m), _ => Arc::new(serde_json::Map::new()) };
    rmcp::model::Tool { name: Cow::Borrowed(name), description: Some(Cow::Borrowed(desc)), input_schema: is, annotations: None, icons: None, meta: None, output_schema: None, title: None }
}
