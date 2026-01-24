//! Image Agent Example
//!
//! An ADK agent that generates and upscales images using natural language.
//!
//! ## Usage
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
use adk_tool::McpToolset;
use anyhow::Result;
use rmcp::{ServiceExt, transport::TokioChildProcess};
use std::sync::Arc;
use tokio::process::Command;

/// Path to the image MCP server binary
const IMAGE_SERVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/release/adk-rust-mcp-image");

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
    dotenvy::dotenv().ok();

    // Get API key for the LLM
    let api_key = std::env::var("GOOGLE_API_KEY")
        .expect("GOOGLE_API_KEY environment variable required");

    // Create the LLM model for the agent
    let model = Arc::new(GeminiModel::new(&api_key, "gemini-2.0-flash")?);

    println!("🖼️  Image Agent");
    println!("================");
    println!("Starting image generation MCP server...");

    // Start the image MCP server
    let server_path = std::path::Path::new(IMAGE_SERVER);
    if !server_path.exists() {
        eprintln!("Error: Image server not found at {}", IMAGE_SERVER);
        eprintln!("Please build the servers first: cargo build --release");
        std::process::exit(1);
    }

    let mut cmd = Command::new(IMAGE_SERVER);
    let client = ().serve(TokioChildProcess::new(&mut cmd)?).await?;
    println!("✓ MCP server connected");

    // Create toolset from the MCP client
    let toolset = McpToolset::new(client)
        .with_name("image-tools");

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
    println!("\nShutting down MCP server...");
    cancel_token.cancel();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    result?;
    Ok(())
}
