//! ADK Rust MCP Composer Server

use adk_rust_mcp_common::{Config, McpServerBuilder, TransportArgs};
use adk_rust_mcp_composer::ComposerServer;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "adk-rust-mcp-composer")]
#[command(about = "MCP server for composite media generation")]
struct Args {
    #[command(flatten)]
    transport: TransportArgs,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("adk-rust-mcp-composer server starting...");
    let args = Args::parse();
    let config = Config::from_env()?;
    let server = ComposerServer::new(config);
    let transport = args.transport.into_transport();
    tracing::info!(transport = %transport, "Starting MCP server");

    McpServerBuilder::new(server)
        .with_transport(transport)
        .run()
        .await?;

    Ok(())
}
