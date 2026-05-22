//! ADK Rust MCP Artist Server

use adk_rust_mcp_artist::ArtistServer;
use adk_rust_mcp_common::{Config, McpServerBuilder, TransportArgs};
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "adk-rust-mcp-artist")]
#[command(about = "MCP server for artistic image creation and style transfer")]
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

    let args = Args::parse();
    let config = Config::from_env()?;
    let server = ArtistServer::new(config);

    McpServerBuilder::new(server)
        .with_transport(args.transport.into_transport())
        .run()
        .await?;

    Ok(())
}
