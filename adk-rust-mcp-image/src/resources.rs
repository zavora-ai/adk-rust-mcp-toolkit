//! MCP Resources for the Image server.
//!
//! This module provides resource implementations for:
//! - `image://models` - List available image generation models
//! - `image://segmentation_classes` - List segmentation classes (Google provider specific)
//! - `image://providers` - List available image providers

use adk_rust_mcp_common::models::IMAGEN_MODELS;
use serde::Serialize;

/// Information about an available image generation model.
#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    /// Model identifier
    pub id: &'static str,
    /// Model aliases
    pub aliases: Vec<&'static str>,
    /// Maximum prompt length in characters
    pub max_prompt_length: usize,
    /// Supported aspect ratios
    pub supported_aspect_ratios: Vec<&'static str>,
    /// Maximum number of images per request
    pub max_images: u8,
}

/// Information about an available image provider.
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

/// Segmentation class for image editing operations.
#[derive(Debug, Clone, Serialize)]
pub struct SegmentationClass {
    /// Class identifier
    pub id: &'static str,
    /// Human-readable name
    pub name: &'static str,
    /// Description of what this class represents
    pub description: &'static str,
}

/// Available segmentation classes for Google Imagen.
/// These are used for image editing operations like inpainting.
pub const SEGMENTATION_CLASSES: &[SegmentationClass] = &[
    SegmentationClass {
        id: "background",
        name: "Background",
        description: "The background of the image",
    },
    SegmentationClass {
        id: "person",
        name: "Person",
        description: "Human figures in the image",
    },
    SegmentationClass {
        id: "face",
        name: "Face",
        description: "Human faces in the image",
    },
    SegmentationClass {
        id: "hair",
        name: "Hair",
        description: "Hair on human figures",
    },
    SegmentationClass {
        id: "clothing",
        name: "Clothing",
        description: "Clothing and accessories on human figures",
    },
    SegmentationClass {
        id: "sky",
        name: "Sky",
        description: "Sky regions in the image",
    },
    SegmentationClass {
        id: "ground",
        name: "Ground",
        description: "Ground or floor surfaces",
    },
    SegmentationClass {
        id: "vegetation",
        name: "Vegetation",
        description: "Plants, trees, and other vegetation",
    },
    SegmentationClass {
        id: "building",
        name: "Building",
        description: "Buildings and architectural structures",
    },
    SegmentationClass {
        id: "vehicle",
        name: "Vehicle",
        description: "Cars, trucks, and other vehicles",
    },
    SegmentationClass {
        id: "animal",
        name: "Animal",
        description: "Animals in the image",
    },
    SegmentationClass {
        id: "food",
        name: "Food",
        description: "Food items in the image",
    },
    SegmentationClass {
        id: "furniture",
        name: "Furniture",
        description: "Furniture and home items",
    },
];

/// List all available image generation models.
pub fn list_models() -> Vec<ModelInfo> {
    IMAGEN_MODELS
        .iter()
        .map(|m| ModelInfo {
            id: m.id,
            aliases: m.aliases.to_vec(),
            max_prompt_length: m.max_prompt_length,
            supported_aspect_ratios: m.supported_aspect_ratios.to_vec(),
            max_images: m.max_images,
        })
        .collect()
}

/// List all available segmentation classes.
pub fn list_segmentation_classes() -> Vec<SegmentationClass> {
    SEGMENTATION_CLASSES.to_vec()
}

/// List all available image providers.
///
/// Currently only Google Imagen is supported, but this will be extended
/// in future phases to include OpenAI DALL-E, local inference, etc.
pub fn list_providers() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            id: "google-imagen".to_string(),
            name: "Google Imagen".to_string(),
            description: "Google's Vertex AI Imagen API for high-quality image generation".to_string(),
            is_default: true,
        },
        // Future providers will be added here:
        // - openai-dalle (Phase 3)
        // - local-flux (Phase 4)
        // - replicate (Phase 5)
    ]
}

/// Get models resource as JSON string.
pub fn models_resource_json() -> String {
    serde_json::to_string_pretty(&list_models()).unwrap_or_else(|_| "[]".to_string())
}

/// Get segmentation classes resource as JSON string.
pub fn segmentation_classes_resource_json() -> String {
    serde_json::to_string_pretty(&list_segmentation_classes()).unwrap_or_else(|_| "[]".to_string())
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
        assert!(model_ids.contains(&"imagen-3.0-generate-002"));
        assert!(model_ids.contains(&"imagen-4.0-generate-preview-06-06"));
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
    fn test_list_segmentation_classes() {
        let classes = list_segmentation_classes();
        assert!(!classes.is_empty());
        
        // Check for some expected classes
        let class_ids: Vec<&str> = classes.iter().map(|c| c.id).collect();
        assert!(class_ids.contains(&"background"));
        assert!(class_ids.contains(&"person"));
        assert!(class_ids.contains(&"face"));
    }

    #[test]
    fn test_list_providers() {
        let providers = list_providers();
        assert!(!providers.is_empty());
        
        // Check that Google Imagen is the default
        let default_provider = providers.iter().find(|p| p.is_default);
        assert!(default_provider.is_some());
        assert_eq!(default_provider.unwrap().id, "google-imagen");
    }

    #[test]
    fn test_models_resource_json() {
        let json = models_resource_json();
        assert!(json.starts_with('['));
        assert!(json.contains("imagen"));
    }

    #[test]
    fn test_segmentation_classes_resource_json() {
        let json = segmentation_classes_resource_json();
        assert!(json.starts_with('['));
        assert!(json.contains("background"));
    }

    #[test]
    fn test_providers_resource_json() {
        let json = providers_resource_json();
        assert!(json.starts_with('['));
        assert!(json.contains("google-imagen"));
    }
}
