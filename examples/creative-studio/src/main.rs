//! Creative Studio Example
//!
//! A multi-agent system with specialized agents for different media types,
//! coordinated by a director agent.
//!
//! ## Usage
//! ```bash
//! cargo run
//! ```
//!
//! ## Example Prompts
//! - "I need a complete brand package: logo, jingle, and video intro"
//! - "Create a podcast intro with music and voice"
//! - "Design a social media video with animated text"

use adk_agent::{LlmAgentBuilder, SequentialAgentBuilder};
use adk_core::{Agent, Content, ReadonlyContext, Toolset};
use adk_model::GeminiModel;
use adk_tool::McpToolset;
use anyhow::Result;
use rmcp::{ServiceExt, transport::TokioChildProcess};
use std::sync::Arc;
use tokio::process::Command;

const IMAGE_SERVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/release/adk-rust-mcp-image");
const VIDEO_SERVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/release/adk-rust-mcp-video");
const MUSIC_SERVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/release/adk-rust-mcp-music");
const SPEECH_SERVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/release/adk-rust-mcp-speech");
const AVTOOL_SERVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/release/adk-rust-mcp-avtool");

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

async fn create_specialist_agent(
    name: &str,
    description: &str,
    instruction: &str,
    server_path: &str,
    model: Arc<GeminiModel>,
    ctx: Arc<dyn ReadonlyContext>,
) -> Result<Option<(Arc<dyn Agent>, tokio_util::sync::CancellationToken)>> {
    let path = std::path::Path::new(server_path);
    if !path.exists() {
        println!("  ⚠️  {} server not found, skipping", name);
        return Ok(None);
    }

    let mut cmd = Command::new(server_path);
    let client = ().serve(TokioChildProcess::new(&mut cmd)?).await?;
    
    let toolset = McpToolset::new(client).with_name(&format!("{}-tools", name));
    let cancel_token = toolset.cancellation_token().await;
    let tools = toolset.tools(ctx).await?;

    let mut builder = LlmAgentBuilder::new(name)
        .description(description)
        .model(model)
        .instruction(instruction);

    for tool in tools {
        builder = builder.tool(tool);
    }

    let agent = builder.build()?;
    println!("  ✓ {} agent ready", name);
    
    Ok(Some((Arc::new(agent), cancel_token)))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = std::env::var("GOOGLE_API_KEY")
        .expect("GOOGLE_API_KEY environment variable required");

    let model = Arc::new(GeminiModel::new(&api_key, "gemini-2.0-flash")?);

    println!("🎨 Creative Studio");
    println!("===================");
    println!("Initializing specialist agents...\n");

    let ctx = Arc::new(SimpleContext) as Arc<dyn ReadonlyContext>;
    let mut cancel_tokens = Vec::new();
    let mut sub_agents: Vec<Arc<dyn Agent>> = Vec::new();

    // Create specialist agents
    let specialists = [
        (
            "visual_artist",
            "Creates images and visual content",
            "You are a visual artist specializing in image generation.\n\
             Use image_generate for creating images and image_upscale for enhancing resolution.\n\
             Focus on visual aesthetics, composition, and style.",
            IMAGE_SERVER,
        ),
        (
            "video_director",
            "Creates and edits video content",
            "You are a video director specializing in video generation.\n\
             Use video_generate for text-to-video and video_from_image for animating stills.\n\
             Consider pacing, motion, and visual storytelling.",
            VIDEO_SERVER,
        ),
        (
            "music_composer",
            "Composes music and soundtracks",
            "You are a music composer creating original compositions.\n\
             Use music_generate to create music from descriptions.\n\
             Consider mood, tempo, instrumentation, and genre.",
            MUSIC_SERVER,
        ),
        (
            "voice_artist",
            "Creates voiceovers and narration",
            "You are a voice artist creating speech and narration.\n\
             Use speech_synthesize for text-to-speech and speech_list_voices to find voices.\n\
             Consider tone, pacing, and emotional delivery.",
            SPEECH_SERVER,
        ),
        (
            "post_producer",
            "Handles media processing and assembly",
            "You are a post-production specialist.\n\
             Use FFmpeg tools to combine, convert, and process media files.\n\
             Handle format conversion, audio mixing, and video assembly.",
            AVTOOL_SERVER,
        ),
    ];

    for (name, desc, instruction, path) in specialists {
        if let Some((agent, token)) = create_specialist_agent(
            name, desc, instruction, path, model.clone(), ctx.clone()
        ).await? {
            sub_agents.push(agent);
            cancel_tokens.push(token);
        }
    }

    if sub_agents.is_empty() {
        eprintln!("\nError: No specialist agents available. Please build the servers first:");
        eprintln!("  cargo build --release");
        std::process::exit(1);
    }

    println!("\n✓ {} specialist agents ready", sub_agents.len());

    // Create the director agent that coordinates specialists
    let director = LlmAgentBuilder::new("creative_director")
        .description("Creative director coordinating specialist agents")
        .model(model.clone())
        .instruction(
            "You are a creative director managing a team of specialists:\n\n\
             - visual_artist: Creates images and graphics\n\
             - video_director: Creates video content\n\
             - music_composer: Composes music and soundtracks\n\
             - voice_artist: Creates voiceovers and narration\n\
             - post_producer: Handles media processing\n\n\
             When users request complex projects:\n\
             1. Break down the project into tasks for each specialist\n\
             2. Coordinate the workflow (e.g., create image → animate → add music)\n\
             3. Delegate to the appropriate specialist agents\n\
             4. Ensure all pieces come together cohesively\n\n\
             Think like a creative director - consider the overall vision,\n\
             brand consistency, and how different media elements work together."
        )
        .sub_agents(sub_agents)
        .build()?;

    println!("\n💬 Chat with the Creative Studio (type 'quit' to exit)\n");

    let result = adk_cli::console::run_console(
        Arc::new(director),
        "studio_session".to_string(),
        "user".to_string(),
    ).await;

    println!("\nShutting down specialist agents...");
    for token in cancel_tokens {
        token.cancel();
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    result?;
    Ok(())
}
