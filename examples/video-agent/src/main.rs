//! Video Agent Example
//!
//! An ADK agent that creates videos from text descriptions.
//!
//! ## Usage
//!
//! First, start the MCP server in one terminal:
//! ```bash
//! cd /path/to/adk-rust-mcp
//! ./target/release/adk-rust-mcp-video --http --port 8081
//! ```
//!
//! Then run the agent in another terminal:
//! ```bash
//! cargo run
//! ```
//!
//! ## Example Prompts
//! - "Create a video of waves crashing on a beach"
//! - "Generate a drone shot flying over a forest"
//! - "Make a video from this image with gentle motion"

use adk_agent::LlmAgentBuilder;
use adk_core::{Content, ReadonlyContext, Toolset};
use adk_model::GeminiModel;
use adk_tool::McpHttpClientBuilder;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

/// Default MCP server endpoint
const DEFAULT_MCP_ENDPOINT: &str = "http://localhost:8081/mcp";

struct SimpleContext;

#[async_trait::async_trait]
impl ReadonlyContext for SimpleContext {
    fn invocation_id(&self) -> &str { "init" }
    fn agent_name(&self) -> &str { "video_agent" }
    fn user_id(&self) -> &str { "user" }
    fn app_name(&self) -> &str { "video_agent" }
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
    let _ = dotenvy::from_filename("../../.env");

    let api_key = std::env::var("GOOGLE_API_KEY")
        .expect("GOOGLE_API_KEY environment variable required");

    let mcp_endpoint = std::env::var("VIDEO_MCP_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_MCP_ENDPOINT.to_string());

    let model = Arc::new(GeminiModel::new(&api_key, "gemini-2.0-flash")?);

    println!("🎬 Video Agent");
    println!("===============");
    println!("Connecting to MCP server at {}...", mcp_endpoint);

    // Connect to the video MCP server via HTTP
    let toolset = McpHttpClientBuilder::new(&mcp_endpoint)
        .timeout(Duration::from_secs(300)) // Video generation can take 5+ minutes
        .connect()
        .await?;

    println!("✓ MCP server connected");

    let cancel_token = toolset.cancellation_token().await;

    let ctx = Arc::new(SimpleContext) as Arc<dyn ReadonlyContext>;
    let tools = toolset.tools(ctx).await?;

    println!("✓ Discovered {} tools:", tools.len());
    for tool in &tools {
        println!("  • {}", tool.name());
    }
    println!();

    let mut builder = LlmAgentBuilder::new("video_agent")
        .description("An AI assistant that generates videos")
        .model(model)
        .instruction(
            "You are a video generation assistant. You can:\n\
             - Generate videos from text prompts using video_generate\n\
             - Create videos from images using video_from_image\n\
             - Extend existing videos using video_extend\n\n\
             Important notes:\n\
             - Video generation requires a GCS URI for output (gs://bucket/path)\n\
             - Generation takes 2-5 minutes - inform the user about wait times\n\
             - Available durations: 4, 6, or 8 seconds\n\
             - Aspect ratios: 16:9 (landscape) or 9:16 (portrait)\n\
             - Veo 3 models support audio generation\n\n\
             Always ask for the GCS bucket if not provided."
        );

    for tool in tools {
        builder = builder.tool(tool);
    }

    let agent = builder.build()?;

    println!("💬 Chat with the video agent (type 'quit' to exit)\n");

    let result = adk_cli::console::run_console(
        Arc::new(agent),
        "video_session".to_string(),
        "user".to_string(),
    ).await;

    println!("\nShutting down...");
    cancel_token.cancel();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    result?;
    Ok(())
}
