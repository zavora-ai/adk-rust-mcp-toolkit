//! MCP Server for educational content generation.

use crate::{explainer, flashcard, quiz, story, whiteboard};
use adk_rust_mcp_common::Config;
use rmcp::{
    model::{CallToolResult, Content, ListResourcesResult, ReadResourceResult, ServerCapabilities, ServerInfo},
    ErrorData as McpError, ServerHandler,
};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone)]
pub struct EducationServer {
    config: Config,
}

impl EducationServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl ServerHandler for EducationServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Educational content generation server. Create whiteboards, flashcards, stories, quizzes, and animated explainers.".into()),
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
                tool("whiteboard_generate", "Generate annotated diagrams, math solutions, and visual explanations as if drawn on a classroom whiteboard.", schema_for!(whiteboard::WhiteboardParams)),
                tool("flashcard_generate", "Generate a set of visual flashcards for studying a topic.", schema_for!(flashcard::FlashcardParams)),
                tool("story_generate", "Generate illustrated children's stories with narration as a video.", schema_for!(story::StoryParams)),
                tool("quiz_generate", "Generate visual multiple-choice quizzes with illustrated options and answer key.", schema_for!(quiz::QuizParams)),
                tool("explainer_generate", "Generate step-by-step animated explanations of concepts as a video.", schema_for!(explainer::ExplainerParams)),
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
                "whiteboard_generate" => {
                    let p: whiteboard::WhiteboardParams = serde_json::from_value(val)
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    whiteboard::generate(&self.config, p).await
                }
                "flashcard_generate" => {
                    let p: flashcard::FlashcardParams = serde_json::from_value(val)
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    flashcard::generate(&self.config, p).await
                }
                "story_generate" => {
                    let p: story::StoryParams = serde_json::from_value(val)
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    story::generate(&self.config, p).await
                }
                "quiz_generate" => {
                    let p: quiz::QuizParams = serde_json::from_value(val)
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    quiz::generate(&self.config, p).await
                }
                "explainer_generate" => {
                    let p: explainer::ExplainerParams = serde_json::from_value(val)
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    explainer::generate(&self.config, p).await
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
