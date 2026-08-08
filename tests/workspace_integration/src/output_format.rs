//! Output format tests.
//!
//! Property 7: Successful Tool Output Format
//! For any successful tool execution, the result SHALL be returned as MCP content
//! (text, image, or embedded data) with appropriate MIME types.
//!
//! **Validates: Requirements 3.11**

use rmcp::model::{CallToolResult, ContentBlock};

/// Validates that a CallToolResult has valid content format.
fn validate_tool_result(result: &CallToolResult) -> Result<(), String> {
    // Result should have content
    if result.content.is_empty() && !result.is_error.unwrap_or(false) {
        return Err("Successful result should have content".to_string());
    }

    // Each content item should have valid structure
    for content in &result.content {
        validate_content(content)?;
    }

    Ok(())
}

/// Validates that a ContentBlock item has valid structure.
fn validate_content(content: &ContentBlock) -> Result<(), String> {
    match content {
        ContentBlock::Text(text_content) => {
            // Text content should have non-empty text
            if text_content.text.is_empty() {
                return Err("Text content should not be empty".to_string());
            }
            Ok(())
        }
        ContentBlock::Image(image_content) => {
            // Image content should have data and mime_type
            if image_content.data.is_empty() {
                return Err("Image content should have data".to_string());
            }
            if image_content.mime_type.is_empty() {
                return Err("Image content should have mime_type".to_string());
            }
            // Validate mime type is an image type
            if !image_content.mime_type.starts_with("image/") {
                return Err(format!(
                    "Image content should have image/* mime type, got: {}",
                    image_content.mime_type
                ));
            }
            Ok(())
        }
        ContentBlock::Audio(audio_content) => {
            // Audio content should have data and mime_type
            if audio_content.data.is_empty() {
                return Err("Audio content should have data".to_string());
            }
            if audio_content.mime_type.is_empty() {
                return Err("Audio content should have mime_type".to_string());
            }
            // Validate mime type is an audio type
            if !audio_content.mime_type.starts_with("audio/") {
                return Err(format!(
                    "Audio content should have audio/* mime type, got: {}",
                    audio_content.mime_type
                ));
            }
            Ok(())
        }
        ContentBlock::Resource(_) => {
            // Resource content is valid
            Ok(())
        }
        ContentBlock::ResourceLink(_) => {
            // Resource link content is valid
            Ok(())
        }
        _ => Err("Unsupported content block variant".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that ContentBlock::text helper creates valid content.
    #[test]
    fn test_content_text_helper() {
        let content = ContentBlock::text("Hello, world!");
        assert!(validate_content(&content).is_ok());
    }

    /// Test that ContentBlock::text with empty string fails validation.
    #[test]
    fn test_content_text_empty_fails() {
        let content = ContentBlock::text("");
        assert!(validate_content(&content).is_err());
    }

    /// Test that ContentBlock::image helper creates valid content.
    #[test]
    fn test_content_image_helper() {
        let content = ContentBlock::image("base64data", "image/png");
        assert!(validate_content(&content).is_ok());
    }

    /// Test that ContentBlock::image with empty data fails validation.
    #[test]
    fn test_content_image_empty_data_fails() {
        let content = ContentBlock::image("", "image/png");
        assert!(validate_content(&content).is_err());
    }

    /// Test that ContentBlock::image with invalid mime type fails validation.
    #[test]
    fn test_content_image_invalid_mime_fails() {
        let content = ContentBlock::image("base64data", "text/plain");
        assert!(validate_content(&content).is_err());
    }

    /// Test that CallToolResult::success helper creates valid result.
    #[test]
    fn test_call_tool_result_success_helper() {
        let result = CallToolResult::success(vec![ContentBlock::text("Success")]);
        assert!(validate_tool_result(&result).is_ok());
        assert!(!result.is_error.unwrap_or(true));
    }

    /// Test that empty content for non-error result fails validation.
    #[test]
    fn test_empty_content_non_error_fails() {
        let mut result = CallToolResult::default();
        result.is_error = Some(false);
        assert!(validate_tool_result(&result).is_err());
    }

    /// Test that empty content for error result is OK.
    #[test]
    fn test_empty_content_error_ok() {
        let mut result = CallToolResult::default();
        result.is_error = Some(true);
        assert!(validate_tool_result(&result).is_ok());
    }

    /// Test that multiple content items are all validated.
    #[test]
    fn test_multiple_content_items_validated() {
        let result = CallToolResult::success(vec![
            ContentBlock::text("First"),
            ContentBlock::text("Second"),
        ]);
        assert!(validate_tool_result(&result).is_ok());
    }

    /// Test that result with one invalid content item fails.
    #[test]
    fn test_one_invalid_content_fails() {
        let result = CallToolResult::success(vec![
            ContentBlock::text("Valid"),
            ContentBlock::text(""), // Invalid
        ]);
        assert!(validate_tool_result(&result).is_err());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: rust-mcp-genmedia, Property 7: Successful Tool Output Format
    // **Validates: Requirements 3.11**
    //
    // For any successful tool execution, the result SHALL be returned as MCP content
    // (text, image, or embedded data) with appropriate MIME types.

    /// Strategy to generate valid text content
    fn valid_text_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 .,!?]{1,100}".prop_filter("Must not be empty", |s| !s.is_empty())
    }

    /// Strategy to generate valid base64 data
    fn valid_base64_data_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9+/=]{4,100}".prop_filter("Must not be empty", |s| !s.is_empty())
    }

    /// Strategy to generate valid image MIME types
    fn valid_image_mime_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("image/png".to_string()),
            Just("image/jpeg".to_string()),
            Just("image/gif".to_string()),
            Just("image/webp".to_string()),
        ]
    }

    /// Strategy to generate valid audio MIME types
    fn valid_audio_mime_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("audio/wav".to_string()),
            Just("audio/mp3".to_string()),
            Just("audio/mpeg".to_string()),
            Just("audio/ogg".to_string()),
        ]
    }

    /// Strategy to generate invalid MIME types for images
    fn invalid_image_mime_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("text/plain".to_string()),
            Just("application/json".to_string()),
            Just("video/mp4".to_string()),
            Just("audio/wav".to_string()),
        ]
    }

    proptest! {
        /// Property 7: Text content with valid text should pass validation
        #[test]
        fn valid_text_content_passes(text in valid_text_strategy()) {
            let content = ContentBlock::text(&text);
            let result = validate_content(&content);
            prop_assert!(result.is_ok(), "Valid text content should pass: {:?}", result.err());
        }

        /// Property 7: Image content with valid data and MIME type should pass
        #[test]
        fn valid_image_content_passes(
            data in valid_base64_data_strategy(),
            mime in valid_image_mime_strategy(),
        ) {
            let content = ContentBlock::image(&data, &mime);
            let result = validate_content(&content);
            prop_assert!(result.is_ok(), "Valid image content should pass: {:?}", result.err());
        }

        /// Property 7: Image content with invalid MIME type should fail
        #[test]
        fn invalid_image_mime_fails(
            data in valid_base64_data_strategy(),
            mime in invalid_image_mime_strategy(),
        ) {
            let content = ContentBlock::image(&data, &mime);
            let result = validate_content(&content);
            prop_assert!(result.is_err(), "Invalid image MIME type should fail");
        }

        /// Property 7: Successful result with valid content should pass
        #[test]
        fn successful_result_with_content_passes(text in valid_text_strategy()) {
            let result = CallToolResult::success(vec![ContentBlock::text(&text)]);
            let validation = validate_tool_result(&result);
            prop_assert!(validation.is_ok(), "Successful result should pass: {:?}", validation.err());
            prop_assert!(!result.is_error.unwrap_or(true), "Should not be marked as error");
        }

        /// Property 7: Multiple valid content items should all pass
        #[test]
        fn multiple_valid_content_passes(
            text1 in valid_text_strategy(),
            text2 in valid_text_strategy(),
        ) {
            let result = CallToolResult::success(vec![
                ContentBlock::text(&text1),
                ContentBlock::text(&text2),
            ]);
            let validation = validate_tool_result(&result);
            prop_assert!(validation.is_ok(), "Multiple valid content should pass");
        }
    }

    /// Property 7: Audio content validation
    /// **Validates: Requirements 3.11**
    #[test]
    fn audio_content_validation() {
        // Valid audio content
        for mime in ["audio/wav", "audio/mp3", "audio/mpeg", "audio/ogg"] {
            let content = ContentBlock::audio("base64audiodata", mime);
            assert!(validate_content(&content).is_ok(), "Audio with {} should be valid", mime);
        }

        // Invalid audio MIME types
        for mime in ["image/png", "text/plain", "video/mp4"] {
            let content = ContentBlock::audio("base64audiodata", mime);
            assert!(validate_content(&content).is_err(), "Audio with {} should be invalid", mime);
        }
    }

    /// Property 7: Empty content validation
    /// **Validates: Requirements 3.11**
    #[test]
    fn empty_content_validation() {
        // Empty text should fail
        let content = ContentBlock::text("");
        assert!(validate_content(&content).is_err(), "Empty text should fail");

        // Empty image data should fail
        let content = ContentBlock::image("", "image/png");
        assert!(validate_content(&content).is_err(), "Empty image data should fail");

        // Empty audio data should fail
        let content = ContentBlock::audio("", "audio/wav");
        assert!(validate_content(&content).is_err(), "Empty audio data should fail");
    }

    /// Property 7: Result structure validation
    /// **Validates: Requirements 3.11**
    #[test]
    fn result_structure_validation() {
        // Success result should have content
        let result = CallToolResult::success(vec![ContentBlock::text("Success")]);
        assert!(validate_tool_result(&result).is_ok());
        assert!(!result.is_error.unwrap_or(true));

        // Empty success result should fail
        let mut result = CallToolResult::default();
        result.is_error = Some(false);
        assert!(validate_tool_result(&result).is_err());

        // Error result can have empty content
        let mut result = CallToolResult::default();
        result.is_error = Some(true);
        assert!(validate_tool_result(&result).is_ok());
    }
}
