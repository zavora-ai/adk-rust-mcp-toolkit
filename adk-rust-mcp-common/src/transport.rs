//! MCP Transport configuration and server builder utilities.
//!
//! This module provides a consistent pattern for configuring and running MCP servers
//! across all GenMedia server crates. It supports three transport modes:
//!
//! - **Stdio**: Default mode for local subprocess communication
//! - **HTTP**: Streamable HTTP transport for web-based clients
//! - **SSE**: Server-Sent Events transport for real-time streaming
//!
//! # Example
//!
//! ```ignore
//! use adk_rust_mcp_common::transport::{Transport, TransportArgs};
//! use clap::Parser;
//!
//! #[derive(Parser)]
//! struct Args {
//!     #[command(flatten)]
//!     transport: TransportArgs,
//! }
//!
//! let args = Args::parse();
//! let transport = args.transport.into_transport();
//! ```

use clap::Args;
use std::fmt;

/// Transport mode for MCP server communication.
///
/// Each transport mode has different characteristics:
/// - `Stdio`: Fast, local-only, full machine access
/// - `Http`: Web-based, scalable, requires network setup
/// - `Sse`: Real-time streaming, web-based
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Transport {
    /// Standard input/output transport (default).
    /// Communicates through stdin/stdout, similar to LSP servers.
    #[default]
    Stdio,
    /// HTTP streamable transport.
    /// Runs on a specified port and accepts HTTP connections.
    Http {
        /// Port to listen on
        port: u16,
    },
    /// Server-Sent Events transport.
    /// Provides real-time streaming over HTTP.
    Sse {
        /// Port to listen on
        port: u16,
    },
}

impl Transport {
    /// Create a new stdio transport.
    pub fn stdio() -> Self {
        Transport::Stdio
    }

    /// Create a new HTTP transport on the specified port.
    pub fn http(port: u16) -> Self {
        Transport::Http { port }
    }

    /// Create a new SSE transport on the specified port.
    pub fn sse(port: u16) -> Self {
        Transport::Sse { port }
    }

    /// Check if this is a stdio transport.
    pub fn is_stdio(&self) -> bool {
        matches!(self, Transport::Stdio)
    }

    /// Check if this is an HTTP transport.
    pub fn is_http(&self) -> bool {
        matches!(self, Transport::Http { .. })
    }

    /// Check if this is an SSE transport.
    pub fn is_sse(&self) -> bool {
        matches!(self, Transport::Sse { .. })
    }

    /// Get the port if this is a network transport.
    pub fn port(&self) -> Option<u16> {
        match self {
            Transport::Stdio => None,
            Transport::Http { port } | Transport::Sse { port } => Some(*port),
        }
    }
}

impl fmt::Display for Transport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Transport::Stdio => write!(f, "stdio"),
            Transport::Http { port } => write!(f, "http (port {})", port),
            Transport::Sse { port } => write!(f, "sse (port {})", port),
        }
    }
}

/// Command-line arguments for transport configuration.
///
/// Use with `clap::Parser` to add transport options to your CLI:
///
/// ```ignore
/// #[derive(Parser)]
/// struct MyArgs {
///     #[command(flatten)]
///     transport: TransportArgs,
/// }
/// ```
#[derive(Args, Debug, Clone)]
pub struct TransportArgs {
    /// Transport mode: stdio, http, or sse
    #[arg(long, default_value = "stdio", value_parser = parse_transport_mode)]
    pub transport: TransportMode,

    /// Port for HTTP/SSE transport (default: 8080, or from PORT env var)
    #[arg(long, env = "PORT", default_value = "8080")]
    pub port: u16,
}

/// Transport mode parsed from command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransportMode {
    #[default]
    Stdio,
    Http,
    Sse,
}

fn parse_transport_mode(s: &str) -> Result<TransportMode, String> {
    match s.to_lowercase().as_str() {
        "stdio" => Ok(TransportMode::Stdio),
        "http" => Ok(TransportMode::Http),
        "sse" => Ok(TransportMode::Sse),
        _ => Err(format!(
            "Invalid transport mode '{}'. Valid options: stdio, http, sse",
            s
        )),
    }
}

impl TransportArgs {
    /// Convert command-line arguments into a Transport configuration.
    pub fn into_transport(self) -> Transport {
        match self.transport {
            TransportMode::Stdio => Transport::Stdio,
            TransportMode::Http => Transport::Http { port: self.port },
            TransportMode::Sse => Transport::Sse { port: self.port },
        }
    }
}

impl Default for TransportArgs {
    fn default() -> Self {
        Self {
            transport: TransportMode::Stdio,
            port: 8080,
        }
    }
}
