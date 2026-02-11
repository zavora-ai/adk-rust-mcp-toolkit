//! ADK Rust MCP Common Library
//!
//! Shared utilities for configuration, GCS operations, model definitions,
//! authentication, error handling, and tracing across all MCP GenMedia servers.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod auth;
pub mod config;
pub mod error;
pub mod gcs;
pub mod models;
pub mod server;
pub mod tracing;
pub mod transport;

#[cfg(feature = "otel")]
#[cfg_attr(docsrs, doc(cfg(feature = "otel")))]
pub mod otel;

#[cfg(test)]
mod config_test;
#[cfg(test)]
mod gcs_test;
#[cfg(test)]
mod auth_test;
#[cfg(test)]
mod error_test;
#[cfg(test)]
mod transport_test;
#[cfg(test)]
mod server_test;
#[cfg(all(test, feature = "otel"))]
mod otel_test;

pub use config::Config;
pub use error::{AuthError, ConfigError, Error, GcsError, GcsOperation, Result};
pub use server::{McpServerBuilder, ServerError, shutdown_channel};
pub use transport::{Transport, TransportArgs, TransportMode};
