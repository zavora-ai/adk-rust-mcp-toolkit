//! MCP Server for diagram generation.

use crate::{from_code, generate, to_code};
use adk_rust_mcp_common::Config;
use rmcp::{
    model::{CallToolResult, Content, ListResourcesResult, ReadResourceResult, ServerCapabilities, ServerInfo},
    ErrorData as McpError, ServerHandler,
};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone)]
pub struct DiagramsServer {
    config: Config,
}

impl DiagramsServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl ServerHandler for DiagramsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Diagram generation server. Generate SVG, Mermaid, and PlantUML diagrams from natural language descriptions.".into()),
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
            use rmcp::model::ListToolsResult;
            use schemars::schema_for;

            let tools = vec![
                tool("diagram_generate", "Generate a diagram from natural language. Uses Gemini to convert description to Mermaid code, then renders to SVG/PNG.", schema_for!(generate::DiagramGenerateParams)),
                tool("diagram_from_code", "Render a diagram from Mermaid or PlantUML source code directly.", schema_for!(from_code::DiagramFromCodeParams)),
                tool("diagram_to_code", "Convert a natural language description to diagram source code (Mermaid or PlantUML) without rendering.", schema_for!(to_code::DiagramToCodeParams)),
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
                "diagram_generate" => {
                    let p: generate::DiagramGenerateParams = serde_json::from_value(val)
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    generate::generate(&self.config, p).await
                }
                "diagram_from_code" => {
                    let p: from_code::DiagramFromCodeParams = serde_json::from_value(val)
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    from_code::from_code(&self.config, p).await
                }
                "diagram_to_code" => {
                    let p: to_code::DiagramToCodeParams = serde_json::from_value(val)
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    to_code::to_code(&self.config, p).await
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
