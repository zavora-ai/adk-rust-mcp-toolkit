//! MCP Resources for the Multimodal server.
//!
//! This module provides resource content for:
//! - `multimodal://language_codes` - Supported language codes for TTS

use crate::handler::{AVAILABLE_VOICES, SUPPORTED_LANGUAGE_CODES};
use serde::{Deserialize, Serialize};

/// Language code entry for the resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageCodeEntry {
    /// Language code (e.g., "en-US")
    pub code: String,
    /// Language name (e.g., "English (US)")
    pub name: String,
}

/// Voice entry for the resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct VoiceEntry {
    /// Voice name
    pub name: String,
    /// Voice description
    pub description: String,
}

/// Get the language codes resource as JSON.
pub fn language_codes_resource_json() -> String {
    let codes: Vec<LanguageCodeEntry> = SUPPORTED_LANGUAGE_CODES
        .iter()
        .map(|&(code, name)| LanguageCodeEntry {
            code: code.to_string(),
            name: name.to_string(),
        })
        .collect();

    serde_json::to_string_pretty(&codes).unwrap_or_else(|_| "[]".to_string())
}

/// Get the voices resource as JSON.
pub fn voices_resource_json() -> String {
    let voices: Vec<VoiceEntry> = AVAILABLE_VOICES
        .iter()
        .map(|&name| VoiceEntry {
            name: name.to_string(),
            description: format!("Gemini TTS voice: {}", name),
        })
        .collect();

    serde_json::to_string_pretty(&voices).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_codes_resource_json() {
        let json = language_codes_resource_json();
        assert!(json.contains("en-US"));
        assert!(json.contains("English (US)"));

        // Verify it's valid JSON
        let parsed: Vec<LanguageCodeEntry> = serde_json::from_str(&json).unwrap();
        assert!(!parsed.is_empty());
    }

    #[test]
    fn test_voices_resource_json() {
        let json = voices_resource_json();
        assert!(json.contains("Kore"));
        assert!(json.contains("Puck"));

        // Verify it's valid JSON
        let parsed: Vec<VoiceEntry> = serde_json::from_str(&json).unwrap();
        assert!(!parsed.is_empty());
    }
}
