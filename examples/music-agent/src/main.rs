//! Music Agent Example
//!
//! An ADK agent that composes music from descriptions.
//!
//! ## Usage
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
use adk_tool::McpToolset;
use anyhow::Result;
use rmcp::{ServiceExt, transport::TokioChildProcess};
use std::sync::Arc;
use tokio::process::Command;

const MUSIC_SERVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/release/adk-rust-mcp-music");

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

    let api_key = std::env::var("GOOGLE_API_KEY")
        .expect("GOOGLE_API_KEY environment variable required");

    let model = Arc::new(GeminiModel::new(&api_key, "gemini-2.0-flash")?);

    println!("🎵 Music Agent");
    println!("===============");
    println!("Starting music generation MCP server...");

    let server_path = std::path::Path::new(MUSIC_SERVER);
    if !server_path.exists() {
        eprintln!("Error: Music server not found at {}", MUSIC_SERVER);
        eprintln!("Please build the servers first: cargo build --release");
        std::process::exit(1);
    }

    let cmd = Command::new(MUSIC_SERVER);
    let client = ().serve(TokioChildProcess::new(cmd)?).await?;
    println!("✓ MCP server connected");

    let toolset = McpToolset::new(client).with_name("music-tools");
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

    println!("\nShutting down MCP server...");
    cancel_token.cancel();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    result?;
    Ok(())
}
