//! ADK Rust MCP Video Server Library
//!
//! This library provides video generation capabilities using Vertex AI Veo API.

pub mod handler;
pub mod resources;
pub mod server;

pub use handler::{VideoT2vParams, VideoI2vParams, VideoGenerateResult, VideoHandler};
pub use server::VideoServer;
