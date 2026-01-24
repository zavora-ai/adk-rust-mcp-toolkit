//! Music Agent Example
//!
//! An ADK agent that composes music from descriptions.
//!
//! ## Usage
//!
//! First, start the MCP server in one terminal:
//! ```bash
//! cd /path/to/adk-rust-mcp
//! ./target/release/adk-rust-mcp-music --http --port 8082
//! ```
//!
//! Then run the agent in another terminal:
//! ```bash
//! cargo run
//! ```
//!
//! ## Example Prompts
//! - "Compose an upbeat jazz piano piece"
//! - "Create ambient electronic music for meditation"
//! - "Generate a rock guitar riff"

use adk_agent::LlmAgentBuilder;
use adk_core::{Content, ReadonlyContext, Toolset};
use adk_model::GeminiModel;
use adk_tool::McpHttpClientBuilder;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

/// Default MCP server endpoint
const DEFAULT_MCP_ENDPOINT: &str = "http://localhost:8082/mcp";

struct SimpleContext;

#[async_trait::async_trait]
impl ReadonlyContext for SimpleContext {
    fn invocation_id(&self) -> &str { "init" }
    fn agent_name(&self) -> &str { "music_agent" }
    fn user_id(&self) -> &str { "user" }
    fn app_name(&self) -> &str { "music_agent" }
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

    let mcp_endpoint = std::env::var("MUSIC_MCP_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_MCP_ENDPOINT.to_string());

    let model = Arc::new(GeminiModel::new(&api_key, "gemini-2.0-flash")?);

    println!("🎵 Music Agent");
    println!("===============");
    println!("Connecting to MCP server at {}...", mcp_endpoint);

    // Connect to the music MCP server via HTTP
    let toolset = McpHttpClientBuilder::new(&mcp_endpoint)
        .timeout(Duration::from_secs(120))
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

    let mut builder = LlmAgentBuilder::new("music_agent")
        .description("An AI assistant that composes music")
        .model(model)
        .instruction(
            "You are a music composition assistant. You can:\n\
             - Generate music from text descriptions using music_generate\n\n\
             Tips for good prompts:\n\
             - Describe the genre, mood, and instruments\n\
             - Use negative_prompt to exclude unwanted elements (e.g., 'vocals, lyrics')\n\
             - You can generate 1-4 variations at once with sample_count\n\n\
             Output options:\n\
             - Save to local file with output_file\n\
             - Upload to GCS with output_gcs_uri\n\
             - Or return base64 data (default)\n\n\
             Help users craft detailed prompts for better results."
        );

    for tool in tools {
        builder = builder.tool(tool);
    }

    let agent = builder.build()?;

    println!("💬 Chat with the music agent (type 'quit' to exit)\n");

    let result = adk_cli::console::run_console(
        Arc::new(agent),
        "music_session".to_string(),
        "user".to_string(),
    ).await;

    println!("\nShutting down...");
    cancel_token.cancel();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    result?;
    Ok(())
}
