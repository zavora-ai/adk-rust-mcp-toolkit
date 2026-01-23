//! MCP Resources for the Video server.
//!
//! This module provides resource implementations for:
//! - `video://models` - List available video generation models
//! - `video://providers` - List available video providers

use adk_rust_mcp_common::models::VEO_MODELS;
use serde::Serialize;

/// Information about an available video generation model.
#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    /// Model identifier
    pub id: &'static str,
    /// Model aliases
    pub aliases: Vec<&'static str>,
    /// Supported aspect ratios
    pub supported_aspect_ratios: Vec<&'static str>,
    /// Supported durations in seconds
    pub supported_durations: Vec<u8>,
    /// Whether the model supports audio generation
    pub supports_audio: bool,
}

/// Information about an available video provider.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderInfo {
    /// Provider identifier
    pub id: String,
    /// Provider display name
    pub name: String,
    /// Provider description
    pub description: String,
    /// Whether this is the default provider
    pub is_default: bool,
}

/// List all available video generation models.
pub fn list_models() -> Vec<ModelInfo> {
    VEO_MODELS
        .iter()
        .map(|m| ModelInfo {
            id: m.id,
            aliases: m.aliases.to_vec(),
            supported_aspect_ratios: m.supported_aspect_ratios.to_vec(),
            supported_durations: m.supported_durations.to_vec(),
            supports_audio: m.supports_audio,
        })
        .collect()
}

/// List all available video providers.
///
/// Currently only Google Veo is supported, but this will be extended
/// in future phases to include Replicate, etc.
pub fn list_providers() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            id: "google-veo".to_string(),
            name: "Google Veo".to_string(),
            description: "Google's Vertex AI Veo API for high-quality video generation".to_string(),
            is_default: true,
        },
        // Future providers will be added here:
        // - replicate (Phase 5)
    ]
}

/// Get models resource as JSON string.
pub fn models_resource_json() -> String {
    serde_json::to_string_pretty(&list_models()).unwrap_or_else(|_| "[]".to_string())
}

/// Get providers resource as JSON string.
pub fn providers_resource_json() -> String {
    serde_json::to_string_pretty(&list_providers()).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_models() {
        let models = list_models();
        assert!(!models.is_empty());
        
        // Check that we have the expected models
        let model_ids: Vec<&str> = models.iter().map(|m| m.id).collect();
        assert!(model_ids.contains(&"veo-2.0-generate-001"));
        assert!(model_ids.contains(&"veo-3.0-generate-preview"));
    }

    #[test]
    fn test_list_models_has_aliases() {
        let models = list_models();
        for model in &models {
            assert!(!model.aliases.is_empty(), "Model {} should have aliases", model.id);
        }
    }

    #[test]
    fn test_list_models_has_aspect_ratios() {
        let models = list_models();
        for model in &models {
            assert!(!model.supported_aspect_ratios.is_empty(), 
                "Model {} should have supported aspect ratios", model.id);
        }
    }

    #[test]
    fn test_veo3_supports_audio() {
        let models = list_models();
        let veo3 = models.iter().find(|m| m.id == "veo-3.0-generate-preview");
        assert!(veo3.is_some());
        assert!(veo3.unwrap().supports_audio);
    }

    #[test]
    fn test_veo2_no_audio() {
        let models = list_models();
        let veo2 = models.iter().find(|m| m.id == "veo-2.0-generate-001");
        assert!(veo2.is_some());
        assert!(!veo2.unwrap().supports_audio);
    }

    #[test]
    fn test_list_providers() {
        let providers = list_providers();
        assert!(!providers.is_empty());
        
        // Check that Google Veo is the default
        let default_provider = providers.iter().find(|p| p.is_default);
        assert!(default_provider.is_some());
        assert_eq!(default_provider.unwrap().id, "google-veo");
    }

    #[test]
    fn test_models_resource_json() {
        let json = models_resource_json();
        assert!(json.starts_with('['));
        assert!(json.contains("veo"));
    }

    #[test]
    fn test_providers_resource_json() {
        let json = providers_resource_json();
        assert!(json.starts_with('['));
        assert!(json.contains("google-veo"));
    }
}
