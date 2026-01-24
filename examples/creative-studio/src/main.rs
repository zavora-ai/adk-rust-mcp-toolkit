//! Creative Studio Example
//!
//! A comprehensive media agent with access to all creative tools,
//! acting as a creative director that can handle complex media projects.
//!
//! ## Usage
//!
//! First, start the MCP servers in separate terminals:
//! ```bash
//! ./target/release/adk-rust-mcp-image --http --port 8080
//! ./target/release/adk-rust-mcp-video --http --port 8081
//! ./target/release/adk-rust-mcp-music --http --port 8082
//! ./target/release/adk-rust-mcp-speech --http --port 8083
//! ./target/release/adk-rust-mcp-avtool --http --port 8084
//! ```
//!
//! Then run the agent:
//! ```bash
//! cargo run
//! ```
//!
//! ## Example Prompts
//! - "I need a complete brand package: logo, jingle, and video intro"
//! - "Create a podcast intro with music and voice"
//! - "Design a social media video with animated text"

use adk_agent::LlmAgentBuilder;
use adk_core::{Content, ReadonlyContext, Toolset};
use adk_model::GeminiModel;
use adk_tool::McpHttpClientBuilder;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

/// MCP server endpoints (can be overridden via environment variables)
const DEFAULT_IMAGE_ENDPOINT: &str = "http://localhost:8080/mcp";
const DEFAULT_VIDEO_ENDPOINT: &str = "http://localhost:8081/mcp";
const DEFAULT_MUSIC_ENDPOINT: &str = "http://localhost:8082/mcp";
const DEFAULT_SPEECH_ENDPOINT: &str = "http://localhost:8083/mcp";
const DEFAULT_AVTOOL_ENDPOINT: &str = "http://localhost:8084/mcp";

struct SimpleContext;

#[async_trait::async_trait]
impl ReadonlyContext for SimpleContext {
    fn invocation_id(&self) -> &str { "init" }
    fn agent_name(&self) -> &str { "creative_studio" }
    fn user_id(&self) -> &str { "user" }
    fn app_name(&self) -> &str { "creative_studio" }
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

    let model = Arc::new(GeminiModel::new(&api_key, "gemini-2.0-flash")?);

    println!("🎨 Creative Studio");
    println!("===================");
    println!("Initializing creative tools...\n");

    let servers = [
        ("Image", std::env::var("IMAGE_MCP_ENDPOINT").unwrap_or_else(|_| DEFAULT_IMAGE_ENDPOINT.to_string()), Duration::from_secs(60)),
        ("Video", std::env::var("VIDEO_MCP_ENDPOINT").unwrap_or_else(|_| DEFAULT_VIDEO_ENDPOINT.to_string()), Duration::from_secs(300)),
        ("Music", std::env::var("MUSIC_MCP_ENDPOINT").unwrap_or_else(|_| DEFAULT_MUSIC_ENDPOINT.to_string()), Duration::from_secs(120)),
        ("Speech", std::env::var("SPEECH_MCP_ENDPOINT").unwrap_or_else(|_| DEFAULT_SPEECH_ENDPOINT.to_string()), Duration::from_secs(60)),
        ("AVTool", std::env::var("AVTOOL_MCP_ENDPOINT").unwrap_or_else(|_| DEFAULT_AVTOOL_ENDPOINT.to_string()), Duration::from_secs(60)),
    ];

    let mut all_tools = Vec::new();
    let mut cancel_tokens = Vec::new();
    let ctx = Arc::new(SimpleContext) as Arc<dyn ReadonlyContext>;

    for (name, endpoint, timeout) in &servers {
        print!("  Loading {} tools ({})... ", name, endpoint);
        
        match McpHttpClientBuilder::new(endpoint)
            .timeout(*timeout)
            .connect()
            .await
        {
            Ok(toolset) => {
                cancel_tokens.push(toolset.cancellation_token().await);
                
                match toolset.tools(ctx.clone()).await {
                    Ok(tools) => {
                        println!("✓ ({} tools)", tools.len());
                        all_tools.extend(tools);
                    }
                    Err(e) => println!("⚠️  failed: {}", e),
                }
            }
            Err(e) => println!("⚠️  failed: {}", e),
        }
    }

    if all_tools.is_empty() {
        eprintln!("\nError: No tools available. Please start the MCP servers first.");
        eprintln!("See the usage instructions at the top of this file.");
        std::process::exit(1);
    }

    println!("\n✓ Total tools available: {}", all_tools.len());
    println!();

    // Build the creative director agent with all tools
    let mut builder = LlmAgentBuilder::new("creative_director")
        .description("Creative director with full media production capabilities")
        .model(model)
        .instruction(
            "You are a creative director with access to a complete media production toolkit.\n\n\
             YOUR CAPABILITIES:\n\n\
             🖼️  VISUAL DESIGN:\n\
             - image_generate: Create images from text descriptions\n\
             - image_upscale: Enhance image resolution (x2, x4)\n\n\
             🎬 VIDEO PRODUCTION:\n\
             - video_generate: Create videos from text (requires GCS output)\n\
             - video_from_image: Animate still images into videos\n\
             - video_extend: Extend existing videos\n\n\
             🎵 AUDIO & MUSIC:\n\
             - music_generate: Compose original music from descriptions\n\
             - speech_synthesize: Convert text to natural speech\n\
             - speech_list_voices: Browse available voice options\n\n\
             🔧 POST-PRODUCTION:\n\
             - ffmpeg_get_media_info: Analyze media files\n\
             - ffmpeg_convert_audio_wav_to_mp3: Convert audio formats\n\
             - ffmpeg_video_to_gif: Create GIFs from videos\n\
             - ffmpeg_combine_audio_and_video: Merge audio/video tracks\n\
             - ffmpeg_overlay_image_on_video: Add watermarks/overlays\n\
             - ffmpeg_concatenate_media_files: Join media files\n\
             - ffmpeg_adjust_volume: Change audio volume\n\
             - ffmpeg_layer_audio_files: Mix multiple audio tracks\n\n\
             WORKFLOW APPROACH:\n\
             1. Understand the creative vision and requirements\n\
             2. Break complex projects into logical steps\n\
             3. Execute each step, building on previous outputs\n\
             4. Combine elements using post-production tools\n\n\
             Think like a creative director - consider brand consistency,\n\
             visual storytelling, and how different media elements work together.\n\n\
             For complex projects, explain your creative plan before executing."
        );

    for tool in all_tools {
        builder = builder.tool(tool);
    }

    let agent = builder.build()?;

    println!("💬 Chat with the Creative Studio (type 'quit' to exit)\n");

    let result = adk_cli::console::run_console(
        Arc::new(agent),
        "studio_session".to_string(),
        "user".to_string(),
    ).await;

    println!("\nShutting down...");
    for token in cancel_tokens {
        token.cancel();
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    result?;
    Ok(())
}
