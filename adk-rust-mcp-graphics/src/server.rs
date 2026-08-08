//! MCP Server for graphic editing.

use crate::{background, edit, enhance, remove, resize};
use adk_rust_mcp_common::Config;
use rmcp::{
    model::{CallToolResult, ContentBlock, ListResourcesResult, ServerCapabilities, ServerInfo},
    ErrorData as McpError, ServerHandler,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct GraphicsServer {
    config: Config,
}

impl GraphicsServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl ServerHandler for GraphicsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("Graphic editing server. Edit images, remove objects, replace backgrounds, resize, and enhance with natural language.")
    }

    fn list_tools(
        &self, _: Option<rmcp::model::PaginatedRequestParams>,
        _: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListToolsResult, McpError>> + Send + '_ {
        async move {
            use rmcp::model::ListToolsResult;
            use schemars::schema_for;
            Ok(ListToolsResult::with_all_items(vec![
                    tool("graphics_edit", "Edit an image with natural language instructions", schema_for!(edit::EditParams)),
                    tool("graphics_remove_object", "Remove an object from an image", schema_for!(remove::RemoveParams)),
                    tool("graphics_replace_background", "Replace the background of an image", schema_for!(background::BackgroundParams)),
                    tool("graphics_resize", "Resize/reframe an image to a new aspect ratio (outpainting)", schema_for!(resize::ResizeParams)),
                    tool("graphics_enhance", "Enhance image quality/details", schema_for!(enhance::EnhanceParams)),
                ]))
        }
    }

    fn call_tool(
        &self, params: rmcp::model::CallToolRequestParams,
        _: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::CallToolResponse, McpError>> + Send + '_ {
        async move {
            let val = serde_json::Value::Object(params.arguments.unwrap_or_default());
            let result = match params.name.as_ref() {
                "graphics_edit" => {
                    let p: edit::EditParams = serde_json::from_value(val).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    edit::generate(&self.config, p).await
                }
                "graphics_remove_object" => {
                    let p: remove::RemoveParams = serde_json::from_value(val).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    remove::generate(&self.config, p).await
                }
                "graphics_replace_background" => {
                    let p: background::BackgroundParams = serde_json::from_value(val).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    background::generate(&self.config, p).await
                }
                "graphics_resize" => {
                    let p: resize::ResizeParams = serde_json::from_value(val).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    resize::generate(&self.config, p).await
                }
                "graphics_enhance" => {
                    let p: enhance::EnhanceParams = serde_json::from_value(val).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    enhance::generate(&self.config, p).await
                }
                _ => return Err(McpError::invalid_params(format!("Unknown tool: {}", params.name), None)),
            };
            match result {
                Ok(msg) => Ok(CallToolResult::success(vec![ContentBlock::text(msg)])),
                Err(e) => Err(McpError::internal_error(e, None)),
            }.map(Into::into)
        }
    }

    fn list_resources(&self, _: Option<rmcp::model::PaginatedRequestParams>, _: rmcp::service::RequestContext<rmcp::service::RoleServer>) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async { Ok(ListResourcesResult::with_all_items(vec![])) }
    }

    fn read_resource(&self, params: rmcp::model::ReadResourceRequestParams, _: rmcp::service::RequestContext<rmcp::service::RoleServer>) -> impl std::future::Future<Output = Result<rmcp::model::ReadResourceResponse, McpError>> + Send + '_ {
        async move { Err(McpError::resource_not_found(format!("Unknown: {}", params.uri), None)) }
    }
}

fn tool(name: &'static str, desc: &'static str, schema: schemars::Schema) -> rmcp::model::Tool {
    let sv = serde_json::to_value(&schema).unwrap_or_default();
    let is = match sv { serde_json::Value::Object(m) => Arc::new(m), _ => Arc::new(serde_json::Map::new()) };
    rmcp::model::Tool::new(name, desc, is)
}
