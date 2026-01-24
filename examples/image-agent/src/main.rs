//! Image Agent Example
//!
//! An ADK agent that generates and upscales images using natural language.
//!
//! ## Usage
//! 
//! First, start the MCP server in one terminal:
//! ```bash
//! cd /path/to/adk-rust-mcp
//! ./target/release/adk-rust-mcp-image --http --port 8080
//! ```
//!
//! Then run the agent in another terminal:
//! ```bash
//! cargo run
//! ```
//!
//! ## Example Prompts
//! - "Generate a beautiful sunset over mountains"
//! - "Create a logo for a tech startup called 'NovaTech'"
//! - "Make a 16:9 landscape of a futuristic city"
//! - "Upscale the last image to 4x resolution"

use adk_agent::LlmAgentBuilder;
use adk_core::{Content, ReadonlyContext, Toolset};
use adk_model::GeminiModel;
use adk_tool::McpHttpClientBuilder;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

/// Default MCP server endpoint
const DEFAULT_MCP_ENDPOINT: &str = "http://localhost:8080/mcp";

/// Minimal context for tool discovery
struct SimpleContext;

#[async_trait::async_trait]
impl ReadonlyContext for SimpleContext {
    fn invocation_id(&self) -> &str { "init" }
    fn agent_name(&self) -> &str { "image_agent" }
    fn user_id(&self) -> &str { "user" }
    fn app_name(&self) -> &str { "image_agent" }
    fn session_id(&self) -> &str { "init" }
    fn branch(&self) -> &str { "main" }
    fn user_content(&self) -> &Content {
        static CONTENT: std::sync::OnceLock<Content> = std::sync::OnceLock::new();
        CONTENT.get_or_init(|| Content::new("user").with_text("init"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env from current dir, then try workspace root
    dotenvy::dotenv().ok();
    let _ = dotenvy::from_filename("../../.env");

    // Get API key for the LLM
    let api_key = std::env::var("GOOGLE_API_KEY")
        .expect("GOOGLE_API_KEY environment variable required");

    // Get MCP server endpoint from env or use default
    let mcp_endpoint = std::env::var("MCP_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_MCP_ENDPOINT.to_string());

    // Create the LLM model for the agent
    let model = Arc::new(GeminiModel::new(&api_key, "gemini-2.0-flash")?);

    println!("🖼️  Image Agent");
    println!("================");
    println!("Connecting to MCP server at {}...", mcp_endpoint);

    // Connect to the image MCP server via HTTP
    let toolset = McpHttpClientBuilder::new(&mcp_endpoint)
        .timeout(Duration::from_secs(60))
        .connect()
        .await?;

    println!("✓ MCP server connected");

    // Get cancellation token for cleanup
    let cancel_token = toolset.cancellation_token().await;

    // Discover available tools
    let ctx = Arc::new(SimpleContext) as Arc<dyn ReadonlyContext>;
    let tools = toolset.tools(ctx).await?;

    println!("✓ Discovered {} tools:", tools.len());
    for tool in &tools {
        println!("  • {}", tool.name());
    }
    println!();

    // Build the agent with MCP tools
    let mut builder = LlmAgentBuilder::new("image_agent")
        .description("An AI assistant that generates and manipulates images")
        .model(model)
        .instruction(
            "You are an image generation assistant. You can:\n\
             - Generate images from text descriptions using image_generate\n\
             - Upscale images to higher resolution using image_upscale\n\n\
             When generating images:\n\
             - Ask for clarification if the prompt is vague\n\
             - Suggest appropriate aspect ratios for the content\n\
             - Offer to save images to files when appropriate\n\n\
             Available aspect ratios: 1:1, 3:4, 4:3, 9:16, 16:9\n\
             You can generate 1-4 images at once."
        );

    for tool in tools {
        builder = builder.tool(tool);
    }

    let agent = builder.build()?;

    println!("💬 Chat with the image agent (type 'quit' to exit)\n");

    // Run interactive console
    let result = adk_cli::console::run_console(
        Arc::new(agent),
        "image_session".to_string(),
        "user".to_string(),
    ).await;

    // Cleanup
    println!("\nShutting down...");
    cancel_token.cancel();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    result?;
    Ok(())
}
