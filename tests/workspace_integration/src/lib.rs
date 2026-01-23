//! Workspace-level integration tests for ADK Rust MCP GenMedia servers.
//!
//! These tests verify:
//! - Each server starts correctly with stdio transport
//! - Tool registration and schema generation
//! - Property-based tests for tool schema validity, input validation, and output format

pub mod server_startup;
pub mod tool_schema;
pub mod input_validation;
pub mod output_format;
