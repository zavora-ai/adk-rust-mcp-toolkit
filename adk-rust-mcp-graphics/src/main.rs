//! ADK Rust MCP Graphics Server

use adk_rust_mcp_common::{Config, McpServerBuilder, TransportArgs};
use adk_rust_mcp_graphics::GraphicsServer;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "adk-rust-mcp-graphics")]
#[command(about = "MCP server for graphic editing with natural language")]
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
    let server = GraphicsServer::new(config);

    McpServerBuilder::new(server)
        .with_transport(args.transport.into_transport())
        .run()
        .await?;

    Ok(())
}
