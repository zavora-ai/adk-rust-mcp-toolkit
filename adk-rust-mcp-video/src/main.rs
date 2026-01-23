//! ADK Rust MCP Video Server
//!
//! MCP server for video generation using Vertex AI Veo API.

use adk_rust_mcp_common::{Config, McpServerBuilder, TransportArgs};
use adk_rust_mcp_video::VideoServer;
use anyhow::Result;
use clap::Parser;

#[cfg(feature = "otel")]
use adk_rust_mcp_common::otel::{init_tracing_with_optional_otel, OtelConfig};

/// Command-line arguments for the video server.
#[derive(Parser, Debug)]
#[command(name = "adk-rust-mcp-video")]
#[command(about = "MCP server for video generation using Vertex AI Veo")]
struct Args {
    /// Transport configuration
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
            .with_service_name("adk-rust-mcp-video");
        init_tracing_with_optional_otel(config).await
    };

    #[cfg(not(feature = "otel"))]
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("adk-rust-mcp-video server starting...");

    // Parse command-line arguments
    let args = Args::parse();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!(
        project_id = %config.project_id,
        location = %config.location,
        "Configuration loaded"
    );

    // Create the server handler
    let server = VideoServer::new(config);

    // Build and run the MCP server
    let transport = args.transport.into_transport();
    tracing::info!(transport = %transport, "Starting MCP server");

    McpServerBuilder::new(server)
        .with_transport(transport)
        .run()
        .await?;

    tracing::info!("Server stopped");
    Ok(())
}
