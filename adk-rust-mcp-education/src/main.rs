//! ADK Rust MCP Education Server

use adk_rust_mcp_common::{Config, McpServerBuilder, TransportArgs};
use adk_rust_mcp_education::EducationServer;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "adk-rust-mcp-education")]
#[command(about = "MCP server for educational content generation")]
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

    tracing::info!("adk-rust-mcp-education server starting...");
    let args = Args::parse();
    let config = Config::from_env()?;
    let server = EducationServer::new(config);
    let transport = args.transport.into_transport();
    tracing::info!(transport = %transport, "Starting MCP server");

    McpServerBuilder::new(server)
        .with_transport(transport)
        .run()
        .await?;

    Ok(())
}
