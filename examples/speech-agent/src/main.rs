//! Speech Agent Example
//!
//! An ADK agent that converts text to natural speech.
//!
//! ## Usage
//!
//! First, start the MCP server in one terminal:
//! ```bash
//! cd /path/to/adk-rust-mcp
//! ./target/release/adk-rust-mcp-speech --http --port 8083
//! ```
//!
//! Then run the agent in another terminal:
//! ```bash
//! cargo run
//! ```
//!
//! ## Example Prompts
//! - "Say 'Welcome to our application' in a cheerful voice"
//! - "Read this paragraph slowly and clearly"
//! - "List the available voices"

use adk_agent::LlmAgentBuilder;
use adk_core::{Content, ReadonlyContext, Toolset};
use adk_model::GeminiModel;
use adk_tool::McpHttpClientBuilder;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

/// Default MCP server endpoint
const DEFAULT_MCP_ENDPOINT: &str = "http://localhost:8083/mcp";

struct SimpleContext;

#[async_trait::async_trait]
impl ReadonlyContext for SimpleContext {
    fn invocation_id(&self) -> &str { "init" }
    fn agent_name(&self) -> &str { "speech_agent" }
    fn user_id(&self) -> &str { "user" }
    fn app_name(&self) -> &str { "speech_agent" }
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

    let mcp_endpoint = std::env::var("SPEECH_MCP_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_MCP_ENDPOINT.to_string());

    let model = Arc::new(GeminiModel::new(&api_key, "gemini-2.0-flash")?);

    println!("🎙️  Speech Agent");
    println!("=================");
    println!("Connecting to MCP server at {}...", mcp_endpoint);

    // Connect to the speech MCP server via HTTP
    let toolset = McpHttpClientBuilder::new(&mcp_endpoint)
        .timeout(Duration::from_secs(60))
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

    let mut builder = LlmAgentBuilder::new("speech_agent")
        .description("An AI assistant that converts text to speech")
        .model(model)
        .instruction(
            "You are a text-to-speech assistant. You can:\n\
             - Convert text to speech using speech_synthesize\n\
             - List available voices using speech_list_voices\n\n\
             Speech parameters:\n\
             - speaking_rate: 0.25 (slow) to 4.0 (fast), default 1.0\n\
             - pitch: -20.0 to +20.0 semitones, default 0.0\n\
             - voice: Chirp3-HD voices available\n\n\
             You can also use custom pronunciations with IPA or X-SAMPA phonetic alphabets.\n\n\
             When users ask you to 'say' something, use speech_synthesize.\n\
             Offer to save audio to files when appropriate."
        );

    for tool in tools {
        builder = builder.tool(tool);
    }

    let agent = builder.build()?;

    println!("💬 Chat with the speech agent (type 'quit' to exit)\n");

    let result = adk_cli::console::run_console(
        Arc::new(agent),
        "speech_session".to_string(),
        "user".to_string(),
    ).await;

    println!("\nShutting down...");
    cancel_token.cancel();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    result?;
    Ok(())
}
