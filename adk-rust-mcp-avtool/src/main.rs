//! ADK Rust MCP AVTool Server
//!
//! MCP server for audio/video processing using FFmpeg.
//!
//! # Tools
//!
//! - `ffmpeg_get_media_info` - Get media file information
//! - `ffmpeg_convert_audio_wav_to_mp3` - Convert WAV to MP3
//! - `ffmpeg_video_to_gif` - Convert video to GIF
//! - `ffmpeg_combine_audio_and_video` - Combine audio and video tracks
//! - `ffmpeg_overlay_image_on_video` - Overlay image on video
//! - `ffmpeg_concatenate_media_files` - Concatenate media files
//! - `ffmpeg_adjust_volume` - Adjust audio volume
//! - `ffmpeg_layer_audio_files` - Layer/mix multiple audio files
//!
//! # Usage
//!
//! ```bash
//! # Run with stdio transport (default)
//! adk-rust-mcp-avtool
//!
//! # Run with HTTP transport
//! adk-rust-mcp-avtool --transport http --port 8080
//!
//! # Run with SSE transport
//! adk-rust-mcp-avtool --transport sse --port 8080
//! ```

use adk_rust_mcp_avtool::AVToolServer;
use adk_rust_mcp_common::{Config, McpServerBuilder, TransportArgs};
use anyhow::Result;
use clap::Parser;

#[cfg(feature = "otel")]
use adk_rust_mcp_common::otel::{init_tracing_with_optional_otel, OtelConfig};

#[derive(Parser, Debug)]
#[command(name = "adk-rust-mcp-avtool")]
#[command(about = "MCP server for audio/video processing using FFmpeg")]
#[command(version)]
struct Args {
    #[command(flatten)]
    transport: TransportArgs,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with optional OpenTelemetry support
    #[cfg(feature = "otel")]
    let _otel_guard = {
        let config = OtelConfig::from_env()
            .unwrap_or_default()
            .with_service_name("adk-rust-mcp-avtool");
        init_tracing_with_optional_otel(config).await
    };

    #[cfg(not(feature = "otel"))]
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Parse command-line arguments
    let args = Args::parse();
    
    // Load configuration
    let config = Config::from_env()?;
    
    tracing::info!(
        project_id = %config.project_id,
        location = %config.location,
        "Starting adk-rust-mcp-avtool server"
    );

    // Create server
    let server = AVToolServer::new(config);
    
    // Get transport configuration
    let transport = args.transport.into_transport();

    // Run server
    McpServerBuilder::new(server)
        .with_transport(transport)
        .run()
        .await?;

    Ok(())
}
