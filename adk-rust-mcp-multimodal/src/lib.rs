//! ADK Rust MCP Multimodal Server Library
//!
//! This library provides multimodal generation capabilities using Google's Gemini API,
//! including image generation and text-to-speech synthesis.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod handler;
pub mod resources;
pub mod server;

pub use handler::{
    GeneratedAudio, GeneratedImage, ImageGenerateResult, LanguageCodeInfo, MultimodalHandler,
    MultimodalImageParams, MultimodalTtsParams, TtsResult, VoiceInfo,
};
pub use server::MultimodalServer;
