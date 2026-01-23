//! Input parameter validation tests.
//!
//! Property 6: Input Parameter Validation
//! For any tool invocation with invalid parameters, the server SHALL return
//! an MCP error response with validation details.
//!
//! **Validates: Requirements 3.9**

#[cfg(test)]
mod tests {
    /// Test that ImageGenerateParams validation rejects invalid parameters.
    /// **Validates: Requirements 3.9**
    #[test]
    fn test_image_params_validation_rejects_invalid() {
        use adk_rust_mcp_image::ImageGenerateParams;

        // Test with invalid number_of_images (out of range)
        let params = ImageGenerateParams {
            prompt: "A cat".to_string(),
            negative_prompt: None,
            model: "imagen-4".to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 10, // Invalid: max is 4
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err(), "Should reject out-of-range number_of_images");
        
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|e| e.field == "number_of_images"),
            "Should have number_of_images validation error"
        );
    }

    /// Test that ImageGenerateParams validation rejects invalid aspect ratio.
    /// **Validates: Requirements 3.9**
    #[test]
    fn test_image_params_validation_rejects_invalid_aspect_ratio() {
        use adk_rust_mcp_image::ImageGenerateParams;

        let params = ImageGenerateParams {
            prompt: "A cat".to_string(),
            negative_prompt: None,
            model: "imagen-4".to_string(),
            aspect_ratio: "2:1".to_string(), // Invalid
            number_of_images: 1,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err(), "Should reject invalid aspect_ratio");
        
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|e| e.field == "aspect_ratio"),
            "Should have aspect_ratio validation error"
        );
    }

    /// Test that ImageGenerateParams validation rejects empty prompt.
    /// **Validates: Requirements 3.9**
    #[test]
    fn test_image_params_validation_rejects_empty_prompt() {
        use adk_rust_mcp_image::ImageGenerateParams;

        let params = ImageGenerateParams {
            prompt: "   ".to_string(), // Empty/whitespace
            negative_prompt: None,
            model: "imagen-4".to_string(),
            aspect_ratio: "1:1".to_string(),
            number_of_images: 1,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err(), "Should reject empty prompt");
        
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|e| e.field == "prompt"),
            "Should have prompt validation error"
        );
    }

    /// Test that VideoT2vParams validation rejects invalid parameters.
    /// **Validates: Requirements 3.9**
    #[test]
    fn test_video_params_validation_rejects_invalid() {
        use adk_rust_mcp_video::VideoT2vParams;

        // Test with invalid duration_seconds (out of range)
        let params = VideoT2vParams {
            prompt: "A sunset".to_string(),
            model: "veo-3".to_string(),
            aspect_ratio: "16:9".to_string(),
            duration_seconds: 100, // Invalid: max is typically 8
            output_gcs_uri: "gs://bucket/video.mp4".to_string(),
            download_local: false,
            local_path: None,
            generate_audio: None,
            seed: None,
        };

        let result = params.validate();
        assert!(result.is_err(), "Should reject out-of-range duration_seconds");
        
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|e| e.field == "duration_seconds"),
            "Should have duration_seconds validation error"
        );
    }

    /// Test that MusicGenerateParams validation rejects invalid parameters.
    /// **Validates: Requirements 3.9**
    #[test]
    fn test_music_params_validation_rejects_invalid() {
        use adk_rust_mcp_music::MusicGenerateParams;

        // Test with invalid sample_count (out of range)
        let params = MusicGenerateParams {
            prompt: "A jazz melody".to_string(),
            negative_prompt: None,
            seed: None,
            sample_count: 10, // Invalid: max is 4
            output_file: None,
            output_gcs_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err(), "Should reject out-of-range sample_count");
        
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|e| e.field == "sample_count"),
            "Should have sample_count validation error"
        );
    }

    /// Test that SpeechSynthesizeParams validation rejects invalid parameters.
    /// **Validates: Requirements 3.9**
    #[test]
    fn test_speech_params_validation_rejects_invalid() {
        use adk_rust_mcp_speech::SpeechSynthesizeParams;

        // Test with invalid speaking_rate (out of range)
        let params = SpeechSynthesizeParams {
            text: "Hello world".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 10.0, // Invalid: max is 4.0
            pitch: 0.0,
            pronunciations: None,
            output_file: None,
        };

        let result = params.validate();
        assert!(result.is_err(), "Should reject out-of-range speaking_rate");
        
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|e| e.field == "speaking_rate"),
            "Should have speaking_rate validation error"
        );
    }

    /// Test that SpeechSynthesizeParams validation rejects invalid pitch.
    /// **Validates: Requirements 3.9**
    #[test]
    fn test_speech_params_validation_rejects_invalid_pitch() {
        use adk_rust_mcp_speech::SpeechSynthesizeParams;

        // Test with invalid pitch (out of range)
        let params = SpeechSynthesizeParams {
            text: "Hello world".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: 50.0, // Invalid: max is 20.0
            pronunciations: None,
            output_file: None,
        };

        let result = params.validate();
        assert!(result.is_err(), "Should reject out-of-range pitch");
        
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|e| e.field == "pitch"),
            "Should have pitch validation error"
        );
    }

    /// Test that valid parameters pass validation.
    /// **Validates: Requirements 3.9**
    #[test]
    fn test_valid_params_pass_validation() {
        use adk_rust_mcp_image::ImageGenerateParams;

        let params = ImageGenerateParams {
            prompt: "A beautiful sunset".to_string(),
            negative_prompt: Some("blurry".to_string()),
            model: "imagen-4".to_string(),
            aspect_ratio: "16:9".to_string(),
            number_of_images: 2,
            seed: Some(42),
            output_file: None,
            output_uri: None,
        };

        let result = params.validate();
        assert!(result.is_ok(), "Valid params should pass validation");
    }

    /// Test that validation collects multiple errors.
    /// **Validates: Requirements 3.9**
    #[test]
    fn test_validation_collects_multiple_errors() {
        use adk_rust_mcp_image::ImageGenerateParams;

        let params = ImageGenerateParams {
            prompt: "   ".to_string(), // Invalid: empty
            negative_prompt: None,
            model: "unknown-model".to_string(), // Invalid: unknown
            aspect_ratio: "2:1".to_string(), // Invalid: not supported
            number_of_images: 10, // Invalid: out of range
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let result = params.validate();
        assert!(result.is_err(), "Should reject multiple invalid params");
        
        let errors = result.unwrap_err();
        assert!(
            errors.len() >= 3,
            "Should have at least 3 validation errors, got {}",
            errors.len()
        );
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    // Feature: rust-mcp-genmedia, Property 6: Input Parameter Validation
    // **Validates: Requirements 3.9**
    //
    // For any tool invocation with invalid parameters, the server SHALL return
    // an MCP error response with validation details.

    /// Strategy to generate valid prompts
    fn valid_prompt_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 ]{1,100}".prop_filter("Must not be empty", |s| !s.trim().is_empty())
    }

    /// Strategy to generate invalid (empty/whitespace) prompts
    fn invalid_prompt_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("".to_string()),
            Just("   ".to_string()),
            Just("\t\n".to_string()),
        ]
    }

    /// Strategy to generate valid number_of_images (1-4)
    fn valid_number_of_images_strategy() -> impl Strategy<Value = u8> {
        1u8..=4u8
    }

    /// Strategy to generate invalid number_of_images (0 or > 4)
    fn invalid_number_of_images_strategy() -> impl Strategy<Value = u8> {
        prop_oneof![
            Just(0u8),
            5u8..=u8::MAX,
        ]
    }

    /// Strategy to generate valid aspect ratios
    fn valid_aspect_ratio_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("1:1".to_string()),
            Just("3:4".to_string()),
            Just("4:3".to_string()),
            Just("9:16".to_string()),
            Just("16:9".to_string()),
        ]
    }

    /// Strategy to generate invalid aspect ratios
    fn invalid_aspect_ratio_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("2:1".to_string()),
            Just("1:2".to_string()),
            Just("invalid".to_string()),
            Just("".to_string()),
        ]
    }

    proptest! {
        /// Property 6: Valid prompts should pass validation
        #[test]
        fn valid_prompts_pass_validation(prompt in valid_prompt_strategy()) {
            use adk_rust_mcp_image::ImageGenerateParams;

            let params = ImageGenerateParams {
                prompt,
                negative_prompt: None,
                model: "imagen-4".to_string(),
                aspect_ratio: "1:1".to_string(),
                number_of_images: 1,
                seed: None,
                output_file: None,
                output_uri: None,
            };

            let result = params.validate();
            prop_assert!(result.is_ok(), "Valid prompt should pass validation");
        }

        /// Property 6: Invalid (empty) prompts should fail validation
        #[test]
        fn invalid_prompts_fail_validation(prompt in invalid_prompt_strategy()) {
            use adk_rust_mcp_image::ImageGenerateParams;

            let params = ImageGenerateParams {
                prompt,
                negative_prompt: None,
                model: "imagen-4".to_string(),
                aspect_ratio: "1:1".to_string(),
                number_of_images: 1,
                seed: None,
                output_file: None,
                output_uri: None,
            };

            let result = params.validate();
            prop_assert!(result.is_err(), "Empty prompt should fail validation");
            
            let errors = result.unwrap_err();
            prop_assert!(
                errors.iter().any(|e| e.field == "prompt"),
                "Should have prompt validation error"
            );
        }

        /// Property 6: Valid number_of_images should pass validation
        #[test]
        fn valid_number_of_images_passes(num in valid_number_of_images_strategy()) {
            use adk_rust_mcp_image::ImageGenerateParams;

            let params = ImageGenerateParams {
                prompt: "A cat".to_string(),
                negative_prompt: None,
                model: "imagen-4".to_string(),
                aspect_ratio: "1:1".to_string(),
                number_of_images: num,
                seed: None,
                output_file: None,
                output_uri: None,
            };

            let result = params.validate();
            prop_assert!(result.is_ok(), "Valid number_of_images {} should pass", num);
        }

        /// Property 6: Invalid number_of_images should fail validation
        #[test]
        fn invalid_number_of_images_fails(num in invalid_number_of_images_strategy()) {
            use adk_rust_mcp_image::ImageGenerateParams;

            let params = ImageGenerateParams {
                prompt: "A cat".to_string(),
                negative_prompt: None,
                model: "imagen-4".to_string(),
                aspect_ratio: "1:1".to_string(),
                number_of_images: num,
                seed: None,
                output_file: None,
                output_uri: None,
            };

            let result = params.validate();
            prop_assert!(result.is_err(), "Invalid number_of_images {} should fail", num);
            
            let errors = result.unwrap_err();
            prop_assert!(
                errors.iter().any(|e| e.field == "number_of_images"),
                "Should have number_of_images validation error"
            );
        }

        /// Property 6: Valid aspect ratios should pass validation
        #[test]
        fn valid_aspect_ratios_pass(ratio in valid_aspect_ratio_strategy()) {
            use adk_rust_mcp_image::ImageGenerateParams;

            let params = ImageGenerateParams {
                prompt: "A cat".to_string(),
                negative_prompt: None,
                model: "imagen-4".to_string(),
                aspect_ratio: ratio.clone(),
                number_of_images: 1,
                seed: None,
                output_file: None,
                output_uri: None,
            };

            let result = params.validate();
            prop_assert!(result.is_ok(), "Valid aspect_ratio '{}' should pass", ratio);
        }

        /// Property 6: Invalid aspect ratios should fail validation
        #[test]
        fn invalid_aspect_ratios_fail(ratio in invalid_aspect_ratio_strategy()) {
            use adk_rust_mcp_image::ImageGenerateParams;

            let params = ImageGenerateParams {
                prompt: "A cat".to_string(),
                negative_prompt: None,
                model: "imagen-4".to_string(),
                aspect_ratio: ratio.clone(),
                number_of_images: 1,
                seed: None,
                output_file: None,
                output_uri: None,
            };

            let result = params.validate();
            prop_assert!(result.is_err(), "Invalid aspect_ratio '{}' should fail", ratio);
            
            let errors = result.unwrap_err();
            prop_assert!(
                errors.iter().any(|e| e.field == "aspect_ratio"),
                "Should have aspect_ratio validation error"
            );
        }

        /// Property 6: Combination of valid parameters should pass
        #[test]
        fn valid_params_combination_passes(
            prompt in valid_prompt_strategy(),
            num in valid_number_of_images_strategy(),
            ratio in valid_aspect_ratio_strategy(),
        ) {
            use adk_rust_mcp_image::ImageGenerateParams;

            let params = ImageGenerateParams {
                prompt,
                negative_prompt: None,
                model: "imagen-4".to_string(),
                aspect_ratio: ratio,
                number_of_images: num,
                seed: None,
                output_file: None,
                output_uri: None,
            };

            let result = params.validate();
            prop_assert!(result.is_ok(), "Valid params combination should pass");
        }
    }

    /// Property 6: Speech params validation for speaking_rate
    /// **Validates: Requirements 3.9**
    #[test]
    fn speech_speaking_rate_validation() {
        use adk_rust_mcp_speech::SpeechSynthesizeParams;

        // Valid speaking rates (0.25 to 4.0)
        for rate in [0.25, 0.5, 1.0, 2.0, 4.0] {
            let params = SpeechSynthesizeParams {
                text: "Hello".to_string(),
                voice: None,
                language_code: "en-US".to_string(),
                speaking_rate: rate,
                pitch: 0.0,
                pronunciations: None,
                output_file: None,
            };
            assert!(params.validate().is_ok(), "speaking_rate {} should be valid", rate);
        }

        // Invalid speaking rates
        for rate in [0.0, 0.1, 5.0, 10.0] {
            let params = SpeechSynthesizeParams {
                text: "Hello".to_string(),
                voice: None,
                language_code: "en-US".to_string(),
                speaking_rate: rate,
                pitch: 0.0,
                pronunciations: None,
                output_file: None,
            };
            let result = params.validate();
            assert!(result.is_err(), "speaking_rate {} should be invalid", rate);
            let errors = result.unwrap_err();
            assert!(errors.iter().any(|e| e.field == "speaking_rate"));
        }
    }

    /// Property 6: Speech params validation for pitch
    /// **Validates: Requirements 3.9**
    #[test]
    fn speech_pitch_validation() {
        use adk_rust_mcp_speech::SpeechSynthesizeParams;

        // Valid pitch values (-20.0 to 20.0)
        for pitch in [-20.0, -10.0, 0.0, 10.0, 20.0] {
            let params = SpeechSynthesizeParams {
                text: "Hello".to_string(),
                voice: None,
                language_code: "en-US".to_string(),
                speaking_rate: 1.0,
                pitch,
                pronunciations: None,
                output_file: None,
            };
            assert!(params.validate().is_ok(), "pitch {} should be valid", pitch);
        }

        // Invalid pitch values
        for pitch in [-30.0, -21.0, 21.0, 50.0] {
            let params = SpeechSynthesizeParams {
                text: "Hello".to_string(),
                voice: None,
                language_code: "en-US".to_string(),
                speaking_rate: 1.0,
                pitch,
                pronunciations: None,
                output_file: None,
            };
            let result = params.validate();
            assert!(result.is_err(), "pitch {} should be invalid", pitch);
            let errors = result.unwrap_err();
            assert!(errors.iter().any(|e| e.field == "pitch"));
        }
    }

    /// Property 6: Music params validation for sample_count
    /// **Validates: Requirements 3.9**
    #[test]
    fn music_sample_count_validation() {
        use adk_rust_mcp_music::MusicGenerateParams;

        // Valid sample_count (1-4)
        for count in 1u8..=4u8 {
            let params = MusicGenerateParams {
                prompt: "A jazz melody".to_string(),
                negative_prompt: None,
                seed: None,
                sample_count: count,
                output_file: None,
                output_gcs_uri: None,
            };
            assert!(params.validate().is_ok(), "sample_count {} should be valid", count);
        }

        // Invalid sample_count
        for count in [0u8, 5, 10] {
            let params = MusicGenerateParams {
                prompt: "A jazz melody".to_string(),
                negative_prompt: None,
                seed: None,
                sample_count: count,
                output_file: None,
                output_gcs_uri: None,
            };
            let result = params.validate();
            assert!(result.is_err(), "sample_count {} should be invalid", count);
            let errors = result.unwrap_err();
            assert!(errors.iter().any(|e| e.field == "sample_count"));
        }
    }
}
