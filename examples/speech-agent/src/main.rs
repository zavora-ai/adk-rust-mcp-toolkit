//! Speech Agent Example
//!
//! An ADK agent that converts text to natural speech.
//!
//! ## Usage
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
use adk_tool::McpToolset;
use anyhow::Result;
use rmcp::{ServiceExt, transport::TokioChildProcess};
use std::sync::Arc;
use tokio::process::Command;

const SPEECH_SERVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/release/adk-rust-mcp-speech");

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

    let api_key = std::env::var("GOOGLE_API_KEY")
        .expect("GOOGLE_API_KEY environment variable required");

    let model = Arc::new(GeminiModel::new(&api_key, "gemini-2.0-flash")?);

    println!("🎙️  Speech Agent");
    println!("=================");
    println!("Starting speech synthesis MCP server...");

    let server_path = std::path::Path::new(SPEECH_SERVER);
    if !server_path.exists() {
        eprintln!("Error: Speech server not found at {}", SPEECH_SERVER);
        eprintln!("Please build the servers first: cargo build --release");
        std::process::exit(1);
    }

    let mut cmd = Command::new(SPEECH_SERVER);
    let client = ().serve(TokioChildProcess::new(&mut cmd)?).await?;
    println!("✓ MCP server connected");

    let toolset = McpToolset::new(client).with_name("speech-tools");
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

    println!("\nShutting down MCP server...");
    cancel_token.cancel();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    result?;
    Ok(())
}
