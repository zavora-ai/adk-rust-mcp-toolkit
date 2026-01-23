//! MCP Server builder utilities.
//!
//! This module provides a consistent pattern for building and running MCP servers
//! with support for multiple transport modes and graceful shutdown.
//!
//! # Example
//!
//! ```ignore
//! use adk_rust_mcp_common::server::McpServerBuilder;
//! use adk_rust_mcp_common::transport::Transport;
//!
//! let handler = MyHandler::new();
//! McpServerBuilder::new(handler)
//!     .with_transport(Transport::stdio())
//!     .run()
//!     .await?;
//! ```

use crate::transport::Transport;
use rmcp::{ServerHandler, ServiceExt};
use thiserror::Error;
use tokio::sync::oneshot;

/// Errors that can occur when running an MCP server.
#[derive(Debug, Error)]
pub enum ServerError {
    /// Failed to bind to the specified port
    #[error("Failed to bind to port {port}: {message}")]
    BindFailed { port: u16, message: String },

    /// Transport error during communication
    #[error("Transport error: {0}")]
    Transport(String),

    /// Server was shut down
    #[error("Server shutdown")]
    Shutdown,

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Builder for configuring and running MCP servers.
///
/// Provides a fluent API for setting up MCP servers with different
/// transport modes and configurations.
pub struct McpServerBuilder<H> {
    handler: H,
    transport: Transport,
    shutdown_rx: Option<oneshot::Receiver<()>>,
}

impl<H> McpServerBuilder<H>
where
    H: ServerHandler + Clone + Send + Sync + 'static,
{
    /// Create a new server builder with the given handler.
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            transport: Transport::default(),
            shutdown_rx: None,
        }
    }

    /// Set the transport mode for the server.
    pub fn with_transport(mut self, transport: Transport) -> Self {
        self.transport = transport;
        self
    }

    /// Set a shutdown signal receiver for graceful shutdown.
    ///
    /// When the sender is dropped or a message is sent, the server
    /// will initiate graceful shutdown.
    pub fn with_shutdown(mut self, shutdown_rx: oneshot::Receiver<()>) -> Self {
        self.shutdown_rx = Some(shutdown_rx);
        self
    }

    /// Run the MCP server with the configured transport.
    ///
    /// This method blocks until the server is shut down (via signal or shutdown channel).
    pub async fn run(self) -> Result<(), ServerError> {
        tracing::info!(transport = %self.transport, "Starting MCP server");

        match self.transport {
            Transport::Stdio => self.run_stdio().await,
            Transport::Http { port } => self.run_http(port).await,
            Transport::Sse { port } => self.run_sse(port).await,
        }
    }

    /// Run the server with stdio transport.
    async fn run_stdio(self) -> Result<(), ServerError> {
        use rmcp::transport::io::stdio;

        let transport = stdio();

        // Set up graceful shutdown
        let shutdown_future = async {
            if let Some(rx) = self.shutdown_rx {
                let _ = rx.await;
            } else {
                // Wait for SIGTERM or SIGINT
                wait_for_shutdown_signal().await;
            }
        };

        // Run the server
        let service = self
            .handler
            .serve(transport)
            .await
            .map_err(|e| ServerError::Transport(e.to_string()))?;

        tokio::select! {
            result = service.waiting() => {
                result.map_err(|e| ServerError::Transport(e.to_string()))?;
                Ok(())
            }
            _ = shutdown_future => {
                tracing::info!("Received shutdown signal, stopping server");
                Ok(())
            }
        }
    }

    /// Run the server with HTTP streamable transport.
    async fn run_http(self, port: u16) -> Result<(), ServerError> {
        use rmcp::transport::streamable_http_server::{
            session::local::LocalSessionManager, StreamableHttpService,
        };

        let handler = self.handler.clone();
        let service = StreamableHttpService::new(
            move || Ok(handler.clone()),
            LocalSessionManager::default().into(),
            Default::default(),
        );

        let router = axum::Router::new().nest_service("/mcp", service);

        let bind_addr = format!("0.0.0.0:{}", port);
        let tcp_listener = tokio::net::TcpListener::bind(&bind_addr)
            .await
            .map_err(|e| ServerError::BindFailed {
                port,
                message: e.to_string(),
            })?;

        tracing::info!(port, "HTTP server listening");

        // Set up graceful shutdown
        let shutdown_future = async {
            if let Some(rx) = self.shutdown_rx {
                let _ = rx.await;
            } else {
                wait_for_shutdown_signal().await;
            }
        };

        axum::serve(tcp_listener, router)
            .with_graceful_shutdown(shutdown_future)
            .await
            .map_err(|e| ServerError::Transport(e.to_string()))?;

        tracing::info!("HTTP server stopped");
        Ok(())
    }

    /// Run the server with SSE transport.
    ///
    /// Note: SSE transport uses the same HTTP infrastructure as streamable HTTP
    /// but with Server-Sent Events for real-time streaming.
    async fn run_sse(self, port: u16) -> Result<(), ServerError> {
        // SSE transport in rmcp 0.13 uses the same streamable HTTP server
        // with SSE-based communication
        self.run_http(port).await
    }
}

/// Wait for a shutdown signal (SIGTERM or SIGINT).
async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to register SIGTERM handler");
        let mut sigint =
            signal(SignalKind::interrupt()).expect("Failed to register SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                tracing::info!("Received SIGTERM");
            }
            _ = sigint.recv() => {
                tracing::info!("Received SIGINT");
            }
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to register Ctrl+C handler");
        tracing::info!("Received Ctrl+C");
    }
}

/// Convenience function to set up graceful shutdown handling.
///
/// Returns a sender that can be used to trigger shutdown programmatically,
/// and a receiver to pass to the server builder.
pub fn shutdown_channel() -> (oneshot::Sender<()>, oneshot::Receiver<()>) {
    oneshot::channel()
}
