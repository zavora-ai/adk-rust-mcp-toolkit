//! ADK Rust MCP Music Server
//!
//! MCP server for music generation using Vertex AI Lyria API.

use adk_rust_mcp_common::{Config, McpServerBuilder, TransportArgs};
use adk_rust_mcp_music::MusicServer;
use anyhow::Result;
use clap::Parser;

#[cfg(feature = "otel")]
use adk_rust_mcp_common::otel::{init_tracing_with_optional_otel, OtelConfig};

#[derive(Parser, Debug)]
#[command(name = "adk-rust-mcp-music")]
#[command(about = "MCP server for music generation using Vertex AI Lyria API")]
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
            .with_service_name("adk-rust-mcp-music");
        init_tracing_with_optional_otel(config).await
    };

    #[cfg(not(feature = "otel"))]
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("adk-rust-mcp-music server starting...");

    let args = Args::parse();
    let config = Config::from_env()?;
    let server = MusicServer::new(config);
    let transport = args.transport.into_transport();

    McpServerBuilder::new(server)
        .with_transport(transport)
        .run()
        .await?;

    Ok(())
}
