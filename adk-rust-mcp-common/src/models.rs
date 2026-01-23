//! Model definitions and registry for Imagen, Veo, and Gemini models.
//!
//! This module provides static model definitions and a registry for resolving
//! model names and aliases to their full definitions.

use serde::Serialize;

/// Imagen model definition.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ImagenModel {
    /// Full model identifier
    pub id: &'static str,
    /// Model aliases for convenience
    #[serde(skip)]
    pub aliases: &'static [&'static str],
    /// Maximum prompt length in characters
    pub max_prompt_length: usize,
    /// Supported aspect ratios
    pub supported_aspect_ratios: &'static [&'static str],
    /// Maximum number of images per request
    pub max_images: u8,
}

/// Veo model definition.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct VeoModel {
    /// Full model identifier
    pub id: &'static str,
    /// Model aliases for convenience
    #[serde(skip)]
    pub aliases: &'static [&'static str],
    /// Supported aspect ratios
    pub supported_aspect_ratios: &'static [&'static str],
    /// Supported durations in seconds (discrete values)
    pub supported_durations: &'static [u8],
    /// Whether the model supports audio generation
    pub supports_audio: bool,
}

/// Gemini model definition for multimodal generation.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct GeminiModel {
    /// Full model identifier
    pub id: &'static str,
    /// Model aliases for convenience
    #[serde(skip)]
    pub aliases: &'static [&'static str],
    /// Whether the model supports image generation
    pub supports_image_generation: bool,
    /// Whether the model supports TTS
    pub supports_tts: bool,
}

/// Lyria model definition for music generation.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct LyriaModel {
    /// Full model identifier
    pub id: &'static str,
    /// Model aliases for convenience
    #[serde(skip)]
    pub aliases: &'static [&'static str],
    /// Maximum number of samples per request
    pub max_samples: u8,
}


// =============================================================================
// Static Model Definitions
// =============================================================================

/// Imagen 3.0 Generate model (stable)
pub const IMAGEN_3_0_GENERATE_002: ImagenModel = ImagenModel {
    id: "imagen-3.0-generate-002",
    aliases: &["imagen-3", "imagen-3.0", "imagen3"],
    max_prompt_length: 480,
    supported_aspect_ratios: &["1:1", "3:4", "4:3", "9:16", "16:9"],
    max_images: 4,
};

/// Imagen 3.0 Fast Generate model
pub const IMAGEN_3_0_FAST_GENERATE_001: ImagenModel = ImagenModel {
    id: "imagen-3.0-fast-generate-001",
    aliases: &["imagen-3-fast", "imagen-3.0-fast"],
    max_prompt_length: 480,
    supported_aspect_ratios: &["1:1", "3:4", "4:3", "9:16", "16:9"],
    max_images: 4,
};

/// Imagen 4.0 Generate Preview model (June 2025)
pub const IMAGEN_4_0_GENERATE_PREVIEW_06_06: ImagenModel = ImagenModel {
    id: "imagen-4.0-generate-preview-06-06",
    aliases: &["imagen-4", "imagen-4.0", "imagen4", "imagen-4-preview"],
    max_prompt_length: 2000,
    supported_aspect_ratios: &["1:1", "3:4", "4:3", "9:16", "16:9"],
    max_images: 4,
};

/// All available Imagen models
pub const IMAGEN_MODELS: &[ImagenModel] = &[
    IMAGEN_3_0_GENERATE_002,
    IMAGEN_3_0_FAST_GENERATE_001,
    IMAGEN_4_0_GENERATE_PREVIEW_06_06,
];

// =============================================================================
// Veo Model Definitions
// =============================================================================

/// Veo 2.0 Generate model (stable)
pub const VEO_2_0_GENERATE_001: VeoModel = VeoModel {
    id: "veo-2.0-generate-001",
    aliases: &["veo-2", "veo-2.0", "veo2"],
    supported_aspect_ratios: &["16:9", "9:16"],
    supported_durations: &[4, 6, 8],
    supports_audio: false,
};

/// Veo 3.0 Generate Preview model
pub const VEO_3_0_GENERATE_PREVIEW: VeoModel = VeoModel {
    id: "veo-3.0-generate-preview",
    aliases: &["veo-3", "veo-3.0", "veo3", "veo-3-preview"],
    supported_aspect_ratios: &["16:9", "9:16"],
    supported_durations: &[4, 6, 8],
    supports_audio: true,
};

/// All available Veo models
pub const VEO_MODELS: &[VeoModel] = &[VEO_2_0_GENERATE_001, VEO_3_0_GENERATE_PREVIEW];

// =============================================================================
// Gemini Model Definitions
// =============================================================================

/// Gemini 2.0 Flash model for multimodal generation
pub const GEMINI_2_0_FLASH: GeminiModel = GeminiModel {
    id: "gemini-2.0-flash",
    aliases: &["gemini-flash", "gemini-2-flash"],
    supports_image_generation: true,
    supports_tts: true,
};

/// Gemini 2.0 Flash Lite model
pub const GEMINI_2_0_FLASH_LITE: GeminiModel = GeminiModel {
    id: "gemini-2.0-flash-lite",
    aliases: &["gemini-flash-lite", "gemini-2-flash-lite"],
    supports_image_generation: true,
    supports_tts: true,
};

/// All available Gemini models
pub const GEMINI_MODELS: &[GeminiModel] = &[GEMINI_2_0_FLASH, GEMINI_2_0_FLASH_LITE];

// =============================================================================
// Lyria Model Definitions
// =============================================================================

/// Lyria 1.0 model for music generation
pub const LYRIA_1_0: LyriaModel = LyriaModel {
    id: "lyria-1.0",
    aliases: &["lyria", "lyria-1", "music-generation"],
    max_samples: 4,
};

/// All available Lyria models
pub const LYRIA_MODELS: &[LyriaModel] = &[LYRIA_1_0];


// =============================================================================
// Model Registry
// =============================================================================

/// Model registry for resolution and listing.
///
/// Provides methods to resolve model names or aliases to their full definitions,
/// and to list all available models.
pub struct ModelRegistry;

impl ModelRegistry {
    /// Resolve an Imagen model name or alias to full model definition.
    ///
    /// Accepts either the canonical model ID (e.g., "imagen-3.0-generate-002")
    /// or any of its aliases (e.g., "imagen-3", "imagen3").
    ///
    /// # Examples
    ///
    /// ```
    /// use adk_rust_mcp_common::models::ModelRegistry;
    ///
    /// // Resolve by canonical ID
    /// let model = ModelRegistry::resolve_imagen("imagen-3.0-generate-002");
    /// assert!(model.is_some());
    ///
    /// // Resolve by alias
    /// let model = ModelRegistry::resolve_imagen("imagen-3");
    /// assert!(model.is_some());
    /// ```
    pub fn resolve_imagen(name: &str) -> Option<&'static ImagenModel> {
        IMAGEN_MODELS
            .iter()
            .find(|model| model.id == name || model.aliases.contains(&name))
    }

    /// Resolve a Veo model name or alias to full model definition.
    ///
    /// Accepts either the canonical model ID (e.g., "veo-2.0-generate-001")
    /// or any of its aliases (e.g., "veo-2", "veo2").
    ///
    /// # Examples
    ///
    /// ```
    /// use adk_rust_mcp_common::models::ModelRegistry;
    ///
    /// // Resolve by canonical ID
    /// let model = ModelRegistry::resolve_veo("veo-2.0-generate-001");
    /// assert!(model.is_some());
    ///
    /// // Resolve by alias
    /// let model = ModelRegistry::resolve_veo("veo-2");
    /// assert!(model.is_some());
    /// ```
    pub fn resolve_veo(name: &str) -> Option<&'static VeoModel> {
        VEO_MODELS
            .iter()
            .find(|model| model.id == name || model.aliases.contains(&name))
    }

    /// Resolve a Gemini model name or alias to full model definition.
    ///
    /// Accepts either the canonical model ID (e.g., "gemini-2.0-flash")
    /// or any of its aliases (e.g., "gemini-flash").
    ///
    /// # Examples
    ///
    /// ```
    /// use adk_rust_mcp_common::models::ModelRegistry;
    ///
    /// // Resolve by canonical ID
    /// let model = ModelRegistry::resolve_gemini("gemini-2.0-flash");
    /// assert!(model.is_some());
    ///
    /// // Resolve by alias
    /// let model = ModelRegistry::resolve_gemini("gemini-flash");
    /// assert!(model.is_some());
    /// ```
    pub fn resolve_gemini(name: &str) -> Option<&'static GeminiModel> {
        GEMINI_MODELS
            .iter()
            .find(|model| model.id == name || model.aliases.contains(&name))
    }

    /// List all available Imagen models.
    ///
    /// # Examples
    ///
    /// ```
    /// use adk_rust_mcp_common::models::ModelRegistry;
    ///
    /// let models = ModelRegistry::list_imagen_models();
    /// assert!(!models.is_empty());
    /// ```
    pub fn list_imagen_models() -> &'static [ImagenModel] {
        IMAGEN_MODELS
    }

    /// List all available Veo models.
    ///
    /// # Examples
    ///
    /// ```
    /// use adk_rust_mcp_common::models::ModelRegistry;
    ///
    /// let models = ModelRegistry::list_veo_models();
    /// assert!(!models.is_empty());
    /// ```
    pub fn list_veo_models() -> &'static [VeoModel] {
        VEO_MODELS
    }

    /// List all available Gemini models.
    ///
    /// # Examples
    ///
    /// ```
    /// use adk_rust_mcp_common::models::ModelRegistry;
    ///
    /// let models = ModelRegistry::list_gemini_models();
    /// assert!(!models.is_empty());
    /// ```
    pub fn list_gemini_models() -> &'static [GeminiModel] {
        GEMINI_MODELS
    }

    /// Resolve a Lyria model name or alias to full model definition.
    ///
    /// Accepts either the canonical model ID (e.g., "lyria-1.0")
    /// or any of its aliases (e.g., "lyria", "music-generation").
    ///
    /// # Examples
    ///
    /// ```
    /// use adk_rust_mcp_common::models::ModelRegistry;
    ///
    /// // Resolve by canonical ID
    /// let model = ModelRegistry::resolve_lyria("lyria-1.0");
    /// assert!(model.is_some());
    ///
    /// // Resolve by alias
    /// let model = ModelRegistry::resolve_lyria("lyria");
    /// assert!(model.is_some());
    /// ```
    pub fn resolve_lyria(name: &str) -> Option<&'static LyriaModel> {
        LYRIA_MODELS
            .iter()
            .find(|model| model.id == name || model.aliases.contains(&name))
    }

    /// List all available Lyria models.
    ///
    /// # Examples
    ///
    /// ```
    /// use adk_rust_mcp_common::models::ModelRegistry;
    ///
    /// let models = ModelRegistry::list_lyria_models();
    /// assert!(!models.is_empty());
    /// ```
    pub fn list_lyria_models() -> &'static [LyriaModel] {
        LYRIA_MODELS
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_imagen_by_id() {
        let model = ModelRegistry::resolve_imagen("imagen-3.0-generate-002");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.id, "imagen-3.0-generate-002");
        assert_eq!(model.max_prompt_length, 480);
    }

    #[test]
    fn test_resolve_imagen_by_alias() {
        let model = ModelRegistry::resolve_imagen("imagen-3");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.id, "imagen-3.0-generate-002");
    }

    #[test]
    fn test_resolve_imagen_4_by_alias() {
        let model = ModelRegistry::resolve_imagen("imagen-4");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.id, "imagen-4.0-generate-preview-06-06");
        assert_eq!(model.max_prompt_length, 2000);
    }

    #[test]
    fn test_resolve_imagen_unknown() {
        let model = ModelRegistry::resolve_imagen("unknown-model");
        assert!(model.is_none());
    }

    #[test]
    fn test_resolve_veo_by_id() {
        let model = ModelRegistry::resolve_veo("veo-2.0-generate-001");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.id, "veo-2.0-generate-001");
        assert!(!model.supports_audio);
    }

    #[test]
    fn test_resolve_veo_by_alias() {
        let model = ModelRegistry::resolve_veo("veo-3");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.id, "veo-3.0-generate-preview");
        assert!(model.supports_audio);
    }

    #[test]
    fn test_resolve_veo_unknown() {
        let model = ModelRegistry::resolve_veo("unknown-model");
        assert!(model.is_none());
    }

    #[test]
    fn test_resolve_gemini_by_id() {
        let model = ModelRegistry::resolve_gemini("gemini-2.0-flash");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.id, "gemini-2.0-flash");
        assert!(model.supports_image_generation);
        assert!(model.supports_tts);
    }

    #[test]
    fn test_resolve_gemini_by_alias() {
        let model = ModelRegistry::resolve_gemini("gemini-flash");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.id, "gemini-2.0-flash");
    }

    #[test]
    fn test_list_imagen_models() {
        let models = ModelRegistry::list_imagen_models();
        assert_eq!(models.len(), 3);
    }

    #[test]
    fn test_list_veo_models() {
        let models = ModelRegistry::list_veo_models();
        assert_eq!(models.len(), 2);
    }

    #[test]
    fn test_list_gemini_models() {
        let models = ModelRegistry::list_gemini_models();
        assert_eq!(models.len(), 2);
    }

    #[test]
    fn test_resolve_lyria_by_id() {
        let model = ModelRegistry::resolve_lyria("lyria-1.0");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.id, "lyria-1.0");
        assert_eq!(model.max_samples, 4);
    }

    #[test]
    fn test_resolve_lyria_by_alias() {
        let model = ModelRegistry::resolve_lyria("lyria");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.id, "lyria-1.0");
    }

    #[test]
    fn test_resolve_lyria_unknown() {
        let model = ModelRegistry::resolve_lyria("unknown-model");
        assert!(model.is_none());
    }

    #[test]
    fn test_list_lyria_models() {
        let models = ModelRegistry::list_lyria_models();
        assert_eq!(models.len(), 1);
    }

    #[test]
    fn test_imagen_model_aspect_ratios() {
        let model = ModelRegistry::resolve_imagen("imagen-3").unwrap();
        assert!(model.supported_aspect_ratios.contains(&"1:1"));
        assert!(model.supported_aspect_ratios.contains(&"16:9"));
        assert!(model.supported_aspect_ratios.contains(&"9:16"));
    }

    #[test]
    fn test_veo_model_supported_durations() {
        let model = ModelRegistry::resolve_veo("veo-2").unwrap();
        assert!(model.supported_durations.contains(&4));
        assert!(model.supported_durations.contains(&6));
        assert!(model.supported_durations.contains(&8));
        assert!(!model.supported_durations.contains(&5)); // 5 is not supported
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: rust-mcp-genmedia, Property 4: Model Alias Resolution Consistency
    // **Validates: Requirements 2.14**
    //
    // For any model alias defined in the Models_Module, resolving the alias SHALL
    // return a valid model definition. For any model's canonical ID, resolving it
    // SHALL return the same model definition as resolving any of its aliases.

    /// Strategy to generate valid Imagen model identifiers (canonical IDs and aliases)
    fn imagen_model_identifier_strategy() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            // Canonical IDs
            Just("imagen-3.0-generate-002"),
            Just("imagen-3.0-fast-generate-001"),
            Just("imagen-4.0-generate-preview-06-06"),
            // Aliases for imagen-3.0-generate-002
            Just("imagen-3"),
            Just("imagen-3.0"),
            Just("imagen3"),
            // Aliases for imagen-3.0-fast-generate-001
            Just("imagen-3-fast"),
            Just("imagen-3.0-fast"),
            // Aliases for imagen-4.0-generate-preview-06-06
            Just("imagen-4"),
            Just("imagen-4.0"),
            Just("imagen4"),
            Just("imagen-4-preview"),
        ]
    }

    /// Strategy to generate valid Veo model identifiers (canonical IDs and aliases)
    fn veo_model_identifier_strategy() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            // Canonical IDs
            Just("veo-2.0-generate-001"),
            Just("veo-3.0-generate-preview"),
            // Aliases for veo-2.0-generate-001
            Just("veo-2"),
            Just("veo-2.0"),
            Just("veo2"),
            // Aliases for veo-3.0-generate-preview
            Just("veo-3"),
            Just("veo-3.0"),
            Just("veo3"),
            Just("veo-3-preview"),
        ]
    }

    /// Strategy to generate valid Gemini model identifiers (canonical IDs and aliases)
    fn gemini_model_identifier_strategy() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            // Canonical IDs
            Just("gemini-2.0-flash"),
            Just("gemini-2.0-flash-lite"),
            // Aliases for gemini-2.0-flash
            Just("gemini-flash"),
            Just("gemini-2-flash"),
            // Aliases for gemini-2.0-flash-lite
            Just("gemini-flash-lite"),
            Just("gemini-2-flash-lite"),
        ]
    }

    proptest! {
        /// Property: Any valid Imagen model identifier (ID or alias) resolves to a model
        #[test]
        fn imagen_alias_resolves_to_model(identifier in imagen_model_identifier_strategy()) {
            let model = ModelRegistry::resolve_imagen(identifier);
            prop_assert!(model.is_some(), "Identifier '{}' should resolve to a model", identifier);
        }

        /// Property: Resolving a canonical Imagen ID returns the same model as resolving any alias
        #[test]
        fn imagen_alias_resolves_to_same_model_as_canonical_id(identifier in imagen_model_identifier_strategy()) {
            let model = ModelRegistry::resolve_imagen(identifier).unwrap();
            let canonical_model = ModelRegistry::resolve_imagen(model.id).unwrap();
            prop_assert_eq!(model.id, canonical_model.id);
            prop_assert_eq!(model.max_prompt_length, canonical_model.max_prompt_length);
            prop_assert_eq!(model.max_images, canonical_model.max_images);
        }

        /// Property: Any valid Veo model identifier (ID or alias) resolves to a model
        #[test]
        fn veo_alias_resolves_to_model(identifier in veo_model_identifier_strategy()) {
            let model = ModelRegistry::resolve_veo(identifier);
            prop_assert!(model.is_some(), "Identifier '{}' should resolve to a model", identifier);
        }

        /// Property: Resolving a canonical Veo ID returns the same model as resolving any alias
        #[test]
        fn veo_alias_resolves_to_same_model_as_canonical_id(identifier in veo_model_identifier_strategy()) {
            let model = ModelRegistry::resolve_veo(identifier).unwrap();
            let canonical_model = ModelRegistry::resolve_veo(model.id).unwrap();
            prop_assert_eq!(model.id, canonical_model.id);
            prop_assert_eq!(model.supported_durations, canonical_model.supported_durations);
            prop_assert_eq!(model.supports_audio, canonical_model.supports_audio);
        }

        /// Property: Any valid Gemini model identifier (ID or alias) resolves to a model
        #[test]
        fn gemini_alias_resolves_to_model(identifier in gemini_model_identifier_strategy()) {
            let model = ModelRegistry::resolve_gemini(identifier);
            prop_assert!(model.is_some(), "Identifier '{}' should resolve to a model", identifier);
        }

        /// Property: Resolving a canonical Gemini ID returns the same model as resolving any alias
        #[test]
        fn gemini_alias_resolves_to_same_model_as_canonical_id(identifier in gemini_model_identifier_strategy()) {
            let model = ModelRegistry::resolve_gemini(identifier).unwrap();
            let canonical_model = ModelRegistry::resolve_gemini(model.id).unwrap();
            prop_assert_eq!(model.id, canonical_model.id);
            prop_assert_eq!(model.supports_image_generation, canonical_model.supports_image_generation);
            prop_assert_eq!(model.supports_tts, canonical_model.supports_tts);
        }
    }
}
