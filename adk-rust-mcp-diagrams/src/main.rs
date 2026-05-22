//! ADK Rust MCP Diagrams Server

use adk_rust_mcp_common::{Config, McpServerBuilder, TransportArgs};
use adk_rust_mcp_diagrams::DiagramsServer;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "adk-rust-mcp-diagrams")]
#[command(about = "MCP server for diagram generation from natural language")]
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

    tracing::info!("adk-rust-mcp-diagrams server starting...");
    let args = Args::parse();
    let config = Config::from_env()?;
    let server = DiagramsServer::new(config);
    let transport = args.transport.into_transport();
    tracing::info!(transport = %transport, "Starting MCP server");

    McpServerBuilder::new(server)
        .with_transport(transport)
        .run()
        .await?;

    Ok(())
}
