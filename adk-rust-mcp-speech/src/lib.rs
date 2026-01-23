//! ADK Rust MCP Speech Server Library
//!
//! This library provides text-to-speech capabilities using Google Cloud TTS Chirp3-HD API.

pub mod handler;
pub mod server;

pub use handler::{
    GeneratedAudio, Pronunciation, SpeechHandler, SpeechSynthesizeParams, SpeechSynthesizeResult,
};
pub use server::SpeechServer;
