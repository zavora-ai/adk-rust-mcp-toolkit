//! ADK Rust MCP Image Server Library
//!
//! This library provides image generation capabilities using Vertex AI Imagen API.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod handler;
pub mod resources;
pub mod server;

pub use handler::{ImageGenerateParams, ImageGenerateResult, ImageHandler, GeneratedImage};
pub use server::ImageServer;
