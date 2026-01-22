# Implementation Plan: Rust MCP GenMedia

## Overview

This implementation plan breaks down the Rust 2024 MCP GenMedia workspace into discrete coding tasks. Each task builds incrementally on previous work, starting with the shared library and progressing through each MCP server. Property-based tests are included to validate correctness properties from the design.

## Tasks

- [ ] 1. Initialize Cargo workspace and project structure
  - Create root `Cargo.toml` with workspace members and shared dependencies
  - Create directory structure for all 7 crates (adk-rust-mcp-common, adk-rust-mcp-image, adk-rust-mcp-video, adk-rust-mcp-music, adk-rust-mcp-speech, adk-rust-mcp-multimodal, adk-rust-mcp-avtool)
  - Configure Rust 2024 edition for all crates
  - Add workspace-level dependencies: tokio, serde, thiserror, anyhow, rmcp, proptest
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [ ] 2. Implement adk-rust-mcp-common configuration module
  - [ ] 2.1 Create Config struct and ConfigError types
    - Implement `Config::from_env()` loading PROJECT_ID (required), LOCATION, GENMEDIA_BUCKET, PORT (optional with defaults)
    - Implement dotenv loading for `.env` file support
    - Implement `vertex_ai_endpoint()` method for API URL construction
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_
  - [ ] 2.2 Write property test for config defaults
    - **Property 1: Configuration Loading with Defaults**
    - **Validates: Requirements 2.1, 2.3, 2.4, 2.5**

- [ ] 3. Implement adk-rust-mcp-common GCS module
  - [ ] 3.1 Create GcsUri struct and parsing
    - Implement `GcsUri::parse()` for `gs://bucket/path` format
    - Implement `GcsUri::to_string()` for formatting
    - Define GcsError enum with InvalidUri, OperationFailed, AuthError variants
    - _Requirements: 2.9, 2.10_
  - [ ] 3.2 Write property test for GCS URI round-trip
    - **Property 2: GCS URI Round-Trip Parsing**
    - **Validates: Requirements 2.9**
  - [ ] 3.3 Implement GcsClient for upload/download
    - Create async `upload()` method using reqwest and auth token
    - Create async `download()` method
    - Create async `exists()` method for checking object existence
    - _Requirements: 2.7, 2.8_
  - [ ] 3.4 Write unit tests for GCS operations with mocked API
    - Test upload success and failure scenarios
    - Test download success and failure scenarios
    - _Requirements: 2.7, 2.8, 2.10_

- [ ] 4. Implement adk-rust-mcp-common models module
  - [ ] 4.1 Define model structs and registry
    - Create ImagenModel struct with id, aliases, max_prompt_length, supported_aspect_ratios, max_images
    - Create VeoModel struct with id, aliases, supported_aspect_ratios, duration_range, supports_audio
    - Define static model definitions for Imagen 3.x/4.x, Veo 2.x/3.x, Gemini models
    - Implement ModelRegistry with resolve_imagen(), resolve_veo(), list methods
    - _Requirements: 2.11, 2.12, 2.13, 2.14_
  - [ ] 4.2 Write property test for model alias resolution
    - **Property 4: Model Alias Resolution Consistency**
    - **Validates: Requirements 2.14**

- [ ] 5. Implement adk-rust-mcp-common auth module
  - [ ] 5.1 Create AuthProvider using ADC
    - Implement `AuthProvider::new()` loading credentials from ADC
    - Implement `get_token()` with automatic refresh
    - Define AuthError enum with NotConfigured, RefreshFailed variants
    - Support GOOGLE_APPLICATION_CREDENTIALS and gcloud user credentials
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5, 11.7_
  - [ ] 5.2 Write property test for token refresh
    - **Property 18: Token Auto-Refresh**
    - **Validates: Requirements 11.7**

- [ ] 6. Implement adk-rust-mcp-common error types
  - [ ] 6.1 Create unified Error enum
    - Define variants: Config, Gcs, Auth, Api, Validation, Io, Ffmpeg, Timeout
    - Implement From traits for error conversion
    - Implement Display with context for API and GCS errors
    - _Requirements: 10.1, 10.2, 10.4, 10.5, 10.6, 10.7_
  - [ ] 6.2 Write property test for error context
    - **Property 16: Error Context Inclusion**
    - **Validates: Requirements 10.4, 10.5**

- [ ] 7. Checkpoint - Verify adk-rust-mcp-common builds and tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 8. Implement MCP server framework utilities
  - [ ] 8.1 Create transport configuration and server builder
    - Define Transport enum (Stdio, Http, Sse)
    - Create McpServerBuilder pattern for consistent server setup
    - Implement command-line argument parsing for --transport and --port
    - Set up graceful shutdown handling for SIGTERM/SIGINT
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 12.1, 12.2, 12.6_
  - [ ] 8.2 Write unit tests for transport configuration
    - Test stdio default, HTTP port binding, SSE endpoint
    - _Requirements: 3.2, 3.3, 3.4, 3.5, 3.6_

- [ ] 9. Implement adk-rust-mcp-image server
  - [ ] 9.1 Create ImageHandler and parameter types
    - Define ImageGenerateParams with serde and JsonSchema derives
    - Implement parameter validation (prompt length, aspect_ratio, number_of_images range)
    - Create ImageHandler struct with config, gcs, http, auth fields
    - _Requirements: 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8_
  - [ ] 9.2 Write property tests for image parameter validation
    - **Property 8: Numeric Parameter Range Validation** (number_of_images)
    - **Property 10: Aspect Ratio Validation**
    - **Validates: Requirements 4.5, 4.6**
  - [ ] 9.3 Implement image_generate tool
    - Build Vertex AI Imagen API request
    - Handle response parsing and base64 image extraction
    - Implement output handling (base64 return, local file, GCS upload)
    - _Requirements: 4.1, 4.9, 4.10, 4.11, 4.14, 4.15_
  - [ ] 9.4 Implement image resources
    - Create image://models resource listing available models
    - Create image://segmentation_classes resource
    - _Requirements: 4.12, 4.13_
  - [ ] 9.5 Wire up adk-rust-mcp-image main.rs
    - Register image_generate tool with rmcp
    - Register resources with rmcp
    - Set up server with transport selection
    - _Requirements: 3.7, 3.8, 3.9, 3.10, 3.11, 3.12_
  - [ ] 9.6 Write unit tests for image handler
    - Test successful generation with mocked API
    - Test error handling for API failures
    - _Requirements: 4.15_

- [ ] 10. Checkpoint - Verify adk-rust-mcp-image builds and tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 11. Implement adk-rust-mcp-video server
  - [ ] 11.1 Create VideoHandler and parameter types
    - Define VideoT2vParams and VideoI2vParams with validation
    - Implement duration_seconds validation based on model
    - Handle generate_audio conditional on Veo 3.x models
    - _Requirements: 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9, 5.10, 5.11, 5.12_
  - [ ] 11.2 Write property tests for video parameter validation
    - **Property 8: Numeric Parameter Range Validation** (duration_seconds)
    - **Property 9: Default Parameter Application**
    - **Validates: Requirements 5.4, 5.6**
  - [ ] 11.3 Implement video_t2v and video_i2v tools
    - Build Vertex AI Veo API request
    - Implement LRO polling with exponential backoff
    - Handle GCS output and optional local download
    - _Requirements: 5.1, 5.2, 5.13, 5.14, 5.15, 5.16_
  - [ ] 11.4 Write property test for LRO polling
    - **Property 11: Long-Running Operation Polling**
    - **Validates: Requirements 5.16**
  - [ ] 11.5 Wire up adk-rust-mcp-video main.rs
    - Register video_t2v and video_i2v tools
    - Set up server with transport selection
    - _Requirements: 3.7, 3.9, 3.11, 3.12_
  - [ ] 11.6 Write unit tests for video handler
    - Test LRO polling with mocked responses
    - Test error handling
    - _Requirements: 5.15, 5.16_

- [ ] 12. Implement adk-rust-mcp-music server
  - [ ] 12.1 Create MusicHandler and parameter types
    - Define MusicGenerateParams with validation
    - Implement sample_count range validation (1-4)
    - _Requirements: 6.2, 6.3, 6.4, 6.5, 6.6, 6.7_
  - [ ] 12.2 Write property test for sample_count validation
    - **Property 8: Numeric Parameter Range Validation** (sample_count)
    - **Validates: Requirements 6.5**
  - [ ] 12.3 Implement music_generate tool
    - Build Vertex AI Lyria API request
    - Handle WAV output (base64, local file, GCS)
    - _Requirements: 6.1, 6.8, 6.9, 6.10, 6.11_
  - [ ] 12.4 Wire up adk-rust-mcp-music main.rs
    - Register music_generate tool
    - Set up server with transport selection
    - _Requirements: 3.7, 3.9, 3.11, 3.12_
  - [ ] 12.5 Write unit tests for music handler
    - Test successful generation with mocked API
    - Test error handling
    - _Requirements: 6.11_

- [ ] 13. Checkpoint - Verify adk-rust-mcp-video and adk-rust-mcp-music build and tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 14. Implement adk-rust-mcp-speech server
  - [ ] 14.1 Create SpeechHandler and parameter types
    - Define SpeechTtsParams with validation
    - Define Pronunciation struct for custom pronunciations
    - Implement speaking_rate (0.25-4.0) and pitch (-20.0 to 20.0) validation
    - _Requirements: 7.3, 7.4, 7.5, 7.6, 7.7, 7.8, 7.10_
  - [ ] 14.2 Write property tests for speech parameter validation
    - **Property 8: Numeric Parameter Range Validation** (speaking_rate, pitch)
    - **Property 12: Pronunciation Alphabet Validation**
    - **Validates: Requirements 7.6, 7.7, 7.9**
  - [ ] 14.3 Implement speech_tts tool
    - Build Cloud TTS API request with Chirp3-HD voice
    - Handle pronunciation SSML generation for IPA/X-SAMPA
    - Handle WAV output (base64, local file)
    - _Requirements: 7.1, 7.9, 7.11, 7.12, 7.14_
  - [ ] 14.4 Implement list_voices tool
    - Query available Chirp3-HD voices
    - Return voice names with language support info
    - _Requirements: 7.2, 7.13_
  - [ ] 14.5 Wire up adk-rust-mcp-speech main.rs
    - Register speech_tts and list_voices tools
    - Set up server with transport selection
    - _Requirements: 3.7, 3.9, 3.11, 3.12_
  - [ ] 14.6 Write unit tests for speech handler
    - Test TTS generation with mocked API
    - Test pronunciation handling
    - _Requirements: 7.9, 7.14_

- [ ] 15. Implement adk-rust-mcp-multimodal server
  - [ ] 15.1 Create MultimodalHandler and parameter types
    - Define MultimodalImageParams and MultimodalTtsParams
    - Implement style/tone parameter handling
    - _Requirements: 8.4, 8.5, 8.6, 8.7, 8.8, 8.9, 8.10_
  - [ ] 15.2 Implement multimodal_image_generation tool
    - Build Gemini API request for image generation
    - Handle output (base64, local file)
    - _Requirements: 8.1, 8.11, 8.13_
  - [ ] 15.3 Implement multimodal_audio_tts tool
    - Build Gemini API request for TTS
    - Handle style/tone control
    - Handle output (base64, local file)
    - _Requirements: 8.2, 8.11, 8.13_
  - [ ] 15.4 Implement list_multimodal_voices tool and language_codes resource
    - Query available Gemini TTS voices
    - Create multimodal://language_codes resource
    - _Requirements: 8.3, 8.12_
  - [ ] 15.5 Wire up adk-rust-mcp-multimodal main.rs
    - Register all tools and resources
    - Set up server with transport selection
    - _Requirements: 3.7, 3.8, 3.9, 3.11, 3.12_
  - [ ] 15.6 Write unit tests for multimodal handler
    - Test image generation with mocked API
    - Test TTS with mocked API
    - _Requirements: 8.13_

- [ ] 16. Checkpoint - Verify adk-rust-mcp-speech and adk-rust-mcp-multimodal build and tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 17. Implement adk-rust-mcp-avtool server
  - [ ] 17.1 Create AVToolHandler and parameter types
    - Define all FFmpeg operation parameter structs (ConvertAudioParams, VideoToGifParams, etc.)
    - Define AudioLayer struct for layering
    - Define MediaInfo and StreamInfo output structs
    - _Requirements: 9.12, 9.13, 9.14, 9.15, 9.16, 9.17, 9.18_
  - [ ] 17.2 Write property test for volume string parsing
    - **Property 15: Volume String Parsing**
    - **Validates: Requirements 9.17**
  - [ ] 17.3 Implement GCS path resolution helpers
    - Create resolve_input() to download GCS URIs to temp files
    - Create handle_output() to upload local files to GCS URIs
    - _Requirements: 9.9, 9.10_
  - [ ] 17.4 Write property test for GCS path resolution
    - **Property 13: GCS Path Resolution**
    - **Validates: Requirements 9.9, 9.10**
  - [ ] 17.5 Implement ffmpeg_get_media_info tool
    - Execute ffprobe and parse JSON output
    - Return MediaInfo with duration, format, streams
    - _Requirements: 9.1, 9.11, 9.20_
  - [ ] 17.6 Write property test for media info output
    - **Property 14: Media Info Output Completeness**
    - **Validates: Requirements 9.11**
  - [ ] 17.7 Implement audio conversion tools
    - Implement ffmpeg_convert_audio_wav_to_mp3
    - Implement ffmpeg_adjust_volume
    - Implement ffmpeg_layer_audio_files
    - _Requirements: 9.2, 9.7, 9.8, 9.19_
  - [ ] 17.8 Implement video processing tools
    - Implement ffmpeg_video_to_gif
    - Implement ffmpeg_combine_audio_and_video
    - Implement ffmpeg_overlay_image_on_video
    - Implement ffmpeg_concatenate_media_files
    - _Requirements: 9.3, 9.4, 9.5, 9.6, 9.19_
  - [ ] 17.9 Wire up adk-rust-mcp-avtool main.rs
    - Register all 8 FFmpeg tools
    - Set up server with transport selection
    - _Requirements: 3.7, 3.9, 3.11, 3.12_
  - [ ] 17.10 Write unit tests for AVTool handler
    - Test media info extraction
    - Test FFmpeg error handling
    - _Requirements: 9.19, 9.20_

- [ ] 18. Checkpoint - Verify adk-rust-mcp-avtool builds and tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 19. Implement OpenTelemetry tracing (optional)
  - [ ] 19.1 Create otel module in adk-rust-mcp-common
    - Implement optional tracing initialization
    - Configure Google Cloud Trace export
    - Add tracing spans to API calls
    - _Requirements: 2.15, 2.16_
  - [ ] 19.2 Write unit tests for otel initialization
    - Test tracing setup when enabled
    - Test graceful handling when disabled
    - _Requirements: 2.15, 2.16_

- [ ] 20. Final integration and validation
  - [ ] 20.1 Add workspace-level integration tests
    - Test each server starts correctly with stdio transport
    - Test tool registration and schema generation
    - _Requirements: 3.7, 3.8_
  - [ ] 20.2 Write property test for tool schema validity
    - **Property 5: Tool Schema Validity**
    - **Validates: Requirements 3.7, 3.8**
  - [ ] 20.3 Write property test for input validation
    - **Property 6: Input Parameter Validation**
    - **Validates: Requirements 3.9**
  - [ ] 20.4 Write property test for output format
    - **Property 7: Successful Tool Output Format**
    - **Validates: Requirements 3.11**
  - [ ] 20.5 Create README.md with usage instructions
    - Document environment variables
    - Document transport options
    - Provide example MCP client configuration
    - _Requirements: 2.1, 3.2, 3.3, 3.4_

- [ ] 21. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

---

## Phase 2: Provider Abstraction (Sprint 3)

- [ ] 22. Extract provider traits from Phase 1 implementations
  - [ ] 22.1 Define core provider traits in adk-rust-mcp-common
    - Create ImageProvider trait with generate(), available_models(), supports() methods
    - Create VideoProvider trait with generate_from_text(), generate_from_image() methods
    - Create SpeechProvider trait with synthesize(), available_voices() methods
    - Create MusicProvider trait with generate() methods
    - Define common output types: ImageOutput, VideoOutput, AudioOutput
    - _Requirements: 13.1, 13.8_
  - [ ] 22.2 Define storage backend trait
    - Create StorageBackend trait with upload(), download(), exists(), get_url(), delete() methods
    - Define StorageError enum
    - _Requirements: 14.1_
  - [ ] 22.3 Implement ProviderRegistry
    - Create registry for managing multiple providers per media type
    - Implement provider selection by name or default
    - Add list_providers() method
    - _Requirements: 13.2, 13.3, 13.4, 13.5, 13.6_
  - [ ] 22.4 Write property tests for provider selection
    - **Property 19: Provider Selection Consistency**
    - **Validates: Requirements 13.3, 13.4, 13.7**

- [ ] 23. Refactor Google Cloud implementations to use traits
  - [ ] 23.1 Create GoogleImagenProvider implementing ImageProvider
    - Extract Imagen API logic from ImageHandler
    - Implement trait methods
    - _Requirements: 15.1_
  - [ ] 23.2 Create GoogleVeoProvider implementing VideoProvider
    - Extract Veo API logic from VideoHandler
    - Implement trait methods
    - _Requirements: 15.2_
  - [ ] 23.3 Create GoogleTtsProvider implementing SpeechProvider
    - Extract Cloud TTS logic from SpeechHandler
    - Implement trait methods
    - _Requirements: 15.3_
  - [ ] 23.4 Create GoogleLyriaProvider implementing MusicProvider
    - Extract Lyria API logic from MusicHandler
    - Implement trait methods
    - _Requirements: 15.4_
  - [ ] 23.5 Wrap GcsClient as GcsStorage implementing StorageBackend
    - Implement StorageBackend trait for GCS
    - _Requirements: 14.2_

- [ ] 24. Refactor handlers to use provider abstraction
  - [ ] 24.1 Update ImageHandler to use ProviderRegistry
    - Accept provider parameter in tool calls
    - Use trait objects for provider calls
    - _Requirements: 13.2, 13.3_
  - [ ] 24.2 Update VideoHandler to use ProviderRegistry
    - Accept provider parameter in tool calls
    - _Requirements: 13.2, 13.3_
  - [ ] 24.3 Update SpeechHandler to use ProviderRegistry
    - Accept provider parameter in tool calls
    - _Requirements: 13.2, 13.3_
  - [ ] 24.4 Update MusicHandler to use ProviderRegistry
    - Accept provider parameter in tool calls
    - _Requirements: 13.2, 13.3_
  - [ ] 24.5 Update AVToolHandler to use StorageBackend trait
    - Replace direct GcsClient usage with StorageBackend
    - _Requirements: 14.5, 14.6, 14.7_

- [ ] 25. Implement LocalStorage backend
  - [ ] 25.1 Create LocalStorage implementing StorageBackend
    - Implement file-based storage operations
    - Support configurable base path
    - _Requirements: 14.4_
  - [ ] 25.2 Write unit tests for LocalStorage
    - Test upload/download round-trip
    - Test path handling
    - _Requirements: 14.4_

- [ ] 26. Add provider configuration
  - [ ] 26.1 Extend Config to support provider selection
    - Add GENMEDIA_PROVIDER_IMAGE, GENMEDIA_PROVIDER_VIDEO, etc. env vars
    - Add GENMEDIA_STORAGE env var for storage backend selection
    - _Requirements: 13.4_
  - [ ] 26.2 Implement provider initialization from config
    - Create providers based on feature flags and configuration
    - Register providers in ProviderRegistry
    - _Requirements: 13.5, 20.1, 20.2_

- [ ] 27. Checkpoint - Verify provider abstraction works with Google Cloud
  - Ensure all existing tests pass with refactored code
  - Verify backward compatibility

---

## Phase 3: OpenAI Provider (Sprint 4)

- [ ] 28. Implement OpenAI image provider
  - [ ] 28.1 Create OpenAIDalleProvider implementing ImageProvider
    - Implement DALL-E 3 API integration
    - Support model selection (dall-e-3, dall-e-2)
    - Handle size/quality parameters
    - _Requirements: 16.1_
  - [ ] 28.2 Add OpenAI configuration
    - Load OPENAI_API_KEY from environment
    - Support optional OPENAI_ORG_ID
    - _Requirements: 16.4, 16.5_
  - [ ] 28.3 Write unit tests for OpenAI image provider
    - Test API request building
    - Test error handling
    - _Requirements: 16.1_

- [ ] 29. Implement OpenAI speech provider
  - [ ] 29.1 Create OpenAITtsProvider implementing SpeechProvider
    - Implement OpenAI TTS API integration
    - Support voice selection (alloy, echo, fable, onyx, nova, shimmer)
    - Handle speed parameter
    - _Requirements: 16.2_
  - [ ] 29.2 Write unit tests for OpenAI speech provider
    - Test API request building
    - Test voice listing
    - _Requirements: 16.2_

- [ ] 30. Implement S3 storage backend (optional)
  - [ ] 30.1 Create S3Storage implementing StorageBackend
    - Use aws-sdk-s3 for S3 operations
    - Support signed URL generation
    - _Requirements: 14.3_
  - [ ] 30.2 Add S3 configuration
    - Load AWS credentials from environment
    - Support bucket and region configuration
    - _Requirements: 14.3_
  - [ ] 30.3 Write unit tests for S3 storage
    - Test with mocked S3 API
    - _Requirements: 14.3_

- [ ] 31. Add feature flags for OpenAI
  - [ ] 31.1 Configure openai feature in Cargo.toml
    - Add async-openai dependency under feature flag
    - Conditionally compile OpenAI providers
    - _Requirements: 20.3_
  - [ ] 31.2 Configure s3 feature in Cargo.toml
    - Add aws-sdk-s3 dependency under feature flag
    - Conditionally compile S3 storage
    - _Requirements: 20.7_

- [ ] 32. Checkpoint - Verify OpenAI provider integration
  - Test with OpenAI API (requires API key)
  - Verify feature flag compilation

---

## Phase 4: Local Inference Provider (Sprint 5-6)

- [ ] 33. Implement mistral.rs image provider
  - [ ] 33.1 Create MistralRsFluxProvider implementing ImageProvider
    - Integrate with adk-mistralrs DiffusionModel
    - Support FLUX.1 Schnell and Dev models
    - Handle device selection (CPU, CUDA, Metal)
    - _Requirements: 17.1, 17.4_
  - [ ] 33.2 Implement model caching and loading
    - Cache models in configurable directory
    - Support model preloading at startup
    - _Requirements: 17.5, 17.6, 17.7_
  - [ ] 33.3 Write unit tests for local image provider
    - Test configuration handling
    - Test model loading (mocked)
    - _Requirements: 17.1_

- [ ] 34. Implement mistral.rs speech provider
  - [ ] 34.1 Create MistralRsDiaProvider implementing SpeechProvider
    - Integrate with adk-mistralrs SpeechModel
    - Support Dia 1.6B model
    - Handle multi-speaker synthesis
    - _Requirements: 17.2_
  - [ ] 34.2 Write unit tests for local speech provider
    - Test configuration handling
    - Test voice listing
    - _Requirements: 17.2_

- [ ] 35. Implement Ollama provider
  - [ ] 35.1 Create OllamaImageProvider implementing ImageProvider
    - Use ollama-rs client
    - Query available models from Ollama API
    - _Requirements: 18.1, 18.4_
  - [ ] 35.2 Add Ollama configuration
    - Support configurable host (default localhost:11434)
    - Handle connection errors gracefully
    - _Requirements: 18.3, 18.5_
  - [ ] 35.3 Write unit tests for Ollama provider
    - Test with mocked Ollama API
    - _Requirements: 18.1_

- [ ] 36. Add feature flags for local inference
  - [ ] 36.1 Configure local feature in Cargo.toml
    - Add adk-mistralrs as git dependency under feature flag
    - Conditionally compile local providers
    - _Requirements: 20.4_
  - [ ] 36.2 Configure ollama feature in Cargo.toml
    - Add ollama-rs dependency under feature flag
    - Conditionally compile Ollama provider
    - _Requirements: 20.5_

- [ ] 37. Checkpoint - Verify local inference providers
  - Test FLUX image generation locally
  - Test Dia speech synthesis locally
  - Test Ollama integration

---

## Phase 5: Additional Providers (Sprint 7+)

- [ ] 38. Implement Replicate provider
  - [ ] 38.1 Create ReplicateImageProvider implementing ImageProvider
    - Implement Replicate predictions API
    - Support SDXL, FLUX, and other models
    - Handle async prediction polling
    - _Requirements: 19.1, 19.5_
  - [ ] 38.2 Create ReplicateVideoProvider implementing VideoProvider
    - Support video generation models on Replicate
    - _Requirements: 19.2_
  - [ ] 38.3 Add Replicate configuration
    - Load REPLICATE_API_TOKEN from environment
    - _Requirements: 19.4, 19.6_
  - [ ] 38.4 Write unit tests for Replicate provider
    - Test prediction API with mocks
    - _Requirements: 19.1, 19.2_

- [ ] 39. Add feature flags for additional providers
  - [ ] 39.1 Configure replicate feature in Cargo.toml
    - Conditionally compile Replicate provider
    - _Requirements: 20.6_
  - [ ] 39.2 Configure all-providers feature
    - Enable all provider features
    - _Requirements: 20.8_

- [ ] 40. Final multi-provider integration
  - [ ] 40.1 Update README with multi-provider documentation
    - Document all supported providers
    - Document feature flags and compilation options
    - Document environment variables for each provider
    - _Requirements: 13.6, 20.1-20.9_
  - [ ] 40.2 Add provider selection examples
    - Show runtime provider selection in tool calls
    - Show configuration-based defaults
    - _Requirements: 13.2, 13.3, 13.4_

- [ ] 41. Final checkpoint - Multi-provider complete
  - Verify all providers work independently
  - Verify provider switching works correctly
  - Ensure feature flag combinations compile correctly

## Notes

- All tasks are required for comprehensive implementation
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The implementation order prioritizes adk-rust-mcp-common first since all servers depend on it
- **Phase 1 (Tasks 1-21)**: Google Cloud provider - can be deployed independently
- **Phase 2 (Tasks 22-27)**: Provider abstraction - refactoring, no new features
- **Phase 3 (Tasks 28-32)**: OpenAI provider - adds cloud alternative
- **Phase 4 (Tasks 33-37)**: Local inference - enables offline operation
- **Phase 5 (Tasks 38-41)**: Additional providers - expands ecosystem
