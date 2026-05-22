//! ADK Rust MCP Diagrams — Generate diagrams from natural language.

pub mod from_code;
pub mod generate;
pub mod server;
pub mod to_code;

pub use server::DiagramsServer;
