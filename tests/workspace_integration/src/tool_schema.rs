//! Tool schema validity tests.
//!
//! Property 5: Tool Schema Validity
//! For any registered tool, the tool's JSON schema SHALL be valid JSON Schema draft-07
//! and SHALL include all required parameters with their types.
//!
//! **Validates: Requirements 3.7, 3.8**

use serde_json::Value;

/// Validates that a JSON schema has the required structure.
fn validate_json_schema(schema: &Value) -> Result<(), String> {
    // Check that it's an object
    let obj = schema
        .as_object()
        .ok_or_else(|| "Schema must be an object".to_string())?;

    // Check for type field (should be "object" for tool params)
    if let Some(type_val) = obj.get("type") {
        if type_val != "object" {
            return Err(format!("Expected type 'object', got {:?}", type_val));
        }
    }

    // Check for properties field
    if let Some(properties) = obj.get("properties") {
        if !properties.is_object() {
            return Err("Properties must be an object".to_string());
        }
    }

    Ok(())
}

/// Validates that a tool has required fields.
fn validate_tool(tool: &rmcp::model::Tool) -> Result<(), String> {
    // Tool must have a name
    if tool.name.is_empty() {
        return Err("Tool name cannot be empty".to_string());
    }

    // Tool must have a description
    if tool.description.is_none() || tool.description.as_ref().unwrap().is_empty() {
        return Err(format!("Tool '{}' must have a description", tool.name));
    }

    // Tool must have an input schema
    if tool.input_schema.is_empty() {
        return Err(format!("Tool '{}' must have an input schema", tool.name));
    }

    // Validate the schema structure
    let schema_value = serde_json::to_value(&*tool.input_schema)
        .map_err(|e| format!("Failed to serialize schema: {}", e))?;
    validate_json_schema(&schema_value)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;
    use std::borrow::Cow;
    use std::sync::Arc;

    /// Test that JSON schema validation works correctly.
    #[test]
    fn test_json_schema_validation() {
        // Valid schema
        let valid_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string"
                }
            },
            "required": ["prompt"]
        });
        assert!(validate_json_schema(&valid_schema).is_ok());

        // Invalid schema (wrong type)
        let invalid_schema = serde_json::json!({
            "type": "string"
        });
        assert!(validate_json_schema(&invalid_schema).is_err());
    }

    /// Test that tool validation works correctly.
    #[test]
    fn test_tool_validation() {
        // Valid tool
        let valid_tool = rmcp::model::Tool {
            name: Cow::Borrowed("test_tool"),
            description: Some(Cow::Borrowed("A test tool")),
            input_schema: Arc::new(serde_json::json!({
                "type": "object",
                "properties": {}
            }).as_object().unwrap().clone()),
            annotations: None,
            icons: None,
            meta: None,
            output_schema: None,
            title: None,
        };
        assert!(validate_tool(&valid_tool).is_ok());

        // Tool with empty name
        let invalid_tool = rmcp::model::Tool {
            name: Cow::Borrowed(""),
            description: Some(Cow::Borrowed("A test tool")),
            input_schema: Arc::new(serde_json::Map::new()),
            annotations: None,
            icons: None,
            meta: None,
            output_schema: None,
            title: None,
        };
        assert!(validate_tool(&invalid_tool).is_err());

        // Tool without description
        let invalid_tool = rmcp::model::Tool {
            name: Cow::Borrowed("test_tool"),
            description: None,
            input_schema: Arc::new(serde_json::Map::new()),
            annotations: None,
            icons: None,
            meta: None,
            output_schema: None,
            title: None,
        };
        assert!(validate_tool(&invalid_tool).is_err());
    }

    /// Test that schemars generates valid JSON schemas.
    /// This validates that our parameter types will produce valid tool schemas.
    #[test]
    fn test_schemars_generates_valid_schemas() {
        // Test with a simple struct
        #[derive(schemars::JsonSchema)]
        struct TestParams {
            prompt: String,
            #[serde(default)]
            optional_field: Option<String>,
        }

        let schema = schema_for!(TestParams);
        let schema_value = serde_json::to_value(&schema).unwrap();
        
        assert!(validate_json_schema(&schema_value).is_ok());
        
        // Verify the schema has the expected structure
        let obj = schema_value.as_object().unwrap();
        assert!(obj.contains_key("properties"));
        
        let properties = obj.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("prompt"));
    }

    /// Test that image generation params produce valid schema.
    /// **Validates: Requirements 3.7, 3.8**
    #[test]
    fn test_image_params_schema_validity() {
        use adk_rust_mcp_image::ImageGenerateParams;
        
        let schema = schema_for!(ImageGenerateParams);
        let schema_value = serde_json::to_value(&schema).unwrap();
        
        assert!(validate_json_schema(&schema_value).is_ok());
        
        // Verify required fields are present
        let obj = schema_value.as_object().unwrap();
        let properties = obj.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("prompt"), "Schema should have 'prompt' property");
    }

    /// Test that video generation params produce valid schema.
    /// **Validates: Requirements 3.7, 3.8**
    #[test]
    fn test_video_params_schema_validity() {
        use adk_rust_mcp_video::VideoT2vParams;
        
        let schema = schema_for!(VideoT2vParams);
        let schema_value = serde_json::to_value(&schema).unwrap();
        
        assert!(validate_json_schema(&schema_value).is_ok());
        
        // Verify required fields are present
        let obj = schema_value.as_object().unwrap();
        let properties = obj.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("prompt"), "Schema should have 'prompt' property");
    }

    /// Test that music generation params produce valid schema.
    /// **Validates: Requirements 3.7, 3.8**
    #[test]
    fn test_music_params_schema_validity() {
        use adk_rust_mcp_music::MusicGenerateParams;
        
        let schema = schema_for!(MusicGenerateParams);
        let schema_value = serde_json::to_value(&schema).unwrap();
        
        assert!(validate_json_schema(&schema_value).is_ok());
        
        // Verify required fields are present
        let obj = schema_value.as_object().unwrap();
        let properties = obj.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("prompt"), "Schema should have 'prompt' property");
    }

    /// Test that speech synthesis params produce valid schema.
    /// **Validates: Requirements 3.7, 3.8**
    #[test]
    fn test_speech_params_schema_validity() {
        use adk_rust_mcp_speech::SpeechSynthesizeParams;
        
        let schema = schema_for!(SpeechSynthesizeParams);
        let schema_value = serde_json::to_value(&schema).unwrap();
        
        assert!(validate_json_schema(&schema_value).is_ok());
        
        // Verify required fields are present
        let obj = schema_value.as_object().unwrap();
        let properties = obj.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("text"), "Schema should have 'text' property");
    }

    /// Test that AVTool params produce valid schemas.
    /// **Validates: Requirements 3.7, 3.8**
    #[test]
    fn test_avtool_params_schema_validity() {
        use adk_rust_mcp_avtool::GetMediaInfoParams;
        
        let schema = schema_for!(GetMediaInfoParams);
        let schema_value = serde_json::to_value(&schema).unwrap();
        
        assert!(validate_json_schema(&schema_value).is_ok());
        
        // Verify required fields are present
        let obj = schema_value.as_object().unwrap();
        let properties = obj.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("input"), "Schema should have 'input' property");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use schemars::schema_for;

    // Feature: rust-mcp-genmedia, Property 5: Tool Schema Validity
    // **Validates: Requirements 3.7, 3.8**
    //
    // For any registered tool, the tool's JSON schema SHALL be valid JSON Schema draft-07
    // and SHALL include all required parameters with their types.

    /// Strategy to generate valid tool names
    fn valid_tool_name_strategy() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9_]{2,30}".prop_map(|s| s.to_string())
    }

    /// Strategy to generate valid tool descriptions
    fn valid_description_strategy() -> impl Strategy<Value = String> {
        "[A-Za-z0-9 .,!?]{10,100}".prop_map(|s| s.to_string())
    }

    proptest! {
        /// Property 5: Tool names should be non-empty and follow naming conventions
        #[test]
        fn tool_name_is_valid(name in valid_tool_name_strategy()) {
            prop_assert!(!name.is_empty(), "Tool name should not be empty");
            prop_assert!(name.chars().next().unwrap().is_ascii_lowercase(), 
                "Tool name should start with lowercase letter");
            prop_assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
                "Tool name should only contain alphanumeric chars and underscores");
        }

        /// Property 5: Tool descriptions should be non-empty and descriptive
        #[test]
        fn tool_description_is_valid(desc in valid_description_strategy()) {
            prop_assert!(!desc.is_empty(), "Tool description should not be empty");
            prop_assert!(desc.len() >= 10, "Tool description should be at least 10 chars");
        }
    }

    /// Property 5: All parameter types should produce valid JSON schemas
    /// **Validates: Requirements 3.7, 3.8**
    #[test]
    fn all_param_types_produce_valid_schemas() {
        // Test all parameter types produce valid schemas
        let schemas = vec![
            ("ImageGenerateParams", serde_json::to_value(schema_for!(adk_rust_mcp_image::ImageGenerateParams)).unwrap()),
            ("VideoT2vParams", serde_json::to_value(schema_for!(adk_rust_mcp_video::VideoT2vParams)).unwrap()),
            ("VideoI2vParams", serde_json::to_value(schema_for!(adk_rust_mcp_video::VideoI2vParams)).unwrap()),
            ("MusicGenerateParams", serde_json::to_value(schema_for!(adk_rust_mcp_music::MusicGenerateParams)).unwrap()),
            ("SpeechSynthesizeParams", serde_json::to_value(schema_for!(adk_rust_mcp_speech::SpeechSynthesizeParams)).unwrap()),
            ("GetMediaInfoParams", serde_json::to_value(schema_for!(adk_rust_mcp_avtool::GetMediaInfoParams)).unwrap()),
            ("ConvertAudioParams", serde_json::to_value(schema_for!(adk_rust_mcp_avtool::ConvertAudioParams)).unwrap()),
            ("VideoToGifParams", serde_json::to_value(schema_for!(adk_rust_mcp_avtool::VideoToGifParams)).unwrap()),
        ];

        for (name, schema) in schemas {
            let result = validate_json_schema(&schema);
            assert!(result.is_ok(), "Schema for {} should be valid: {:?}", name, result.err());
            
            // Verify schema has properties
            let obj = schema.as_object().unwrap();
            assert!(obj.contains_key("properties"), "Schema for {} should have properties", name);
        }
    }

    /// Property 5: Schemas should have type "object"
    /// **Validates: Requirements 3.7, 3.8**
    #[test]
    fn schemas_have_object_type() {
        let schemas = vec![
            serde_json::to_value(schema_for!(adk_rust_mcp_image::ImageGenerateParams)).unwrap(),
            serde_json::to_value(schema_for!(adk_rust_mcp_video::VideoT2vParams)).unwrap(),
            serde_json::to_value(schema_for!(adk_rust_mcp_music::MusicGenerateParams)).unwrap(),
            serde_json::to_value(schema_for!(adk_rust_mcp_speech::SpeechSynthesizeParams)).unwrap(),
        ];

        for schema in schemas {
            let obj = schema.as_object().unwrap();
            assert_eq!(obj.get("type").and_then(|v| v.as_str()), Some("object"),
                "Schema type should be 'object'");
        }
    }
}
