//! ADK Rust MCP Music Server Library
//!
//! This library provides music generation capabilities using Vertex AI Lyria API.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod handler;
pub mod server;

pub use handler::{MusicGenerateParams, MusicGenerateResult, MusicHandler, GeneratedAudio};
pub use server::MusicServer;
