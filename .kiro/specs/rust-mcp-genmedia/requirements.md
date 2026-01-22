# Requirements Document

## Introduction

This document specifies the requirements for a Rust 2024 edition port of the Google Cloud Platform mcp-genmedia-go MCP servers. The project provides Model Context Protocol (MCP) servers for Google Cloud's generative media APIs, enabling AI assistants to generate images, videos, music, speech, and process audio/video content through a standardized protocol.

The Rust implementation will be organized as a Cargo workspace with separate crates for each MCP server tool, plus a shared common library for configuration, GCS utilities, and model definitions.

### Crate Naming Convention

To avoid trademark conflicts with Google product names, the crates use generic descriptive names:

| Function | Crate Name | Binary Name |
|----------|------------|-------------|
| Shared library | adk-rust-mcp-common | (library) |
| Image generation | adk-rust-mcp-image | adk-rust-mcp-image |
| Video generation | adk-rust-mcp-video | adk-rust-mcp-video |
| Music generation | adk-rust-mcp-music | adk-rust-mcp-music |
| Text-to-speech | adk-rust-mcp-speech | adk-rust-mcp-speech |
| Multimodal generation | adk-rust-mcp-multimodal | adk-rust-mcp-multimodal |
| AV processing | adk-rust-mcp-avtool | adk-rust-mcp-avtool |

## Glossary

- **MCP**: Model Context Protocol - A standardized protocol for AI assistant tool integration
- **MCP_Server**: A server implementing the MCP protocol that exposes tools, resources, and prompts
- **Tool**: An MCP callable function that performs an action and returns results
- **Resource**: An MCP queryable data source that provides information
- **Imagen_API**: Google's text-to-image generation API (Vertex AI)
- **Veo_API**: Google's video generation API (Vertex AI)
- **Lyria_API**: Google's music generation API (Vertex AI)
- **Chirp3_API**: Google's text-to-speech API (Cloud TTS)
- **Gemini_API**: Google's multimodal AI API (Vertex AI)
- **GCS**: Google Cloud Storage
- **ADC**: Application Default Credentials - Google Cloud authentication mechanism
- **Vertex_AI**: Google Cloud's managed ML platform for generative AI APIs
- **FFmpeg**: Open-source multimedia processing tool
- **FFprobe**: FFmpeg utility for media file analysis
- **rmcp**: Rust MCP SDK crate for implementing MCP servers
- **Workspace**: Cargo workspace containing multiple related crates
- **Base64_Data**: Binary data encoded as base64 string for JSON transport

## Requirements

### Requirement 1: Workspace Structure and Build System

**User Story:** As a developer, I want a well-organized Cargo workspace structure, so that I can build, test, and maintain each MCP server independently while sharing common code.

#### Acceptance Criteria

1. THE Workspace SHALL use Rust 2024 edition for all crates
2. THE Workspace SHALL contain a root `Cargo.toml` defining workspace members: adk-rust-mcp-common, adk-rust-mcp-image, adk-rust-mcp-video, adk-rust-mcp-music, adk-rust-mcp-speech, adk-rust-mcp-multimodal, adk-rust-mcp-avtool
3. WHEN building the workspace, THE Build_System SHALL compile all crates with shared dependencies resolved at workspace level
4. THE Workspace SHALL define common dependencies (tokio, serde, thiserror, anyhow, rmcp) in workspace `Cargo.toml` for version consistency
5. WHEN a crate depends on adk-rust-mcp-common, THE Build_System SHALL resolve it as a path dependency within the workspace

### Requirement 2: Common Library (adk-rust-mcp-common)

**User Story:** As a developer, I want shared utilities for configuration, GCS operations, and model definitions, so that all MCP servers use consistent implementations.

#### Acceptance Criteria

1. THE Config_Module SHALL load PROJECT_ID from environment variables or `.env` file
2. IF PROJECT_ID is not set, THEN THE Config_Module SHALL return a descriptive error
3. THE Config_Module SHALL load optional LOCATION environment variable with default value "us-central1"
4. THE Config_Module SHALL load optional GENMEDIA_BUCKET environment variable for GCS output
5. THE Config_Module SHALL load optional PORT environment variable for HTTP transport
6. WHEN a `.env` file exists in the working directory, THE Config_Module SHALL load environment variables from it using dotenv
7. THE GCS_Module SHALL provide functions to upload bytes to a GCS bucket path
8. THE GCS_Module SHALL provide functions to download bytes from a GCS URI
9. THE GCS_Module SHALL parse GCS URIs in format `gs://bucket/path/to/object`
10. IF a GCS operation fails, THEN THE GCS_Module SHALL return an error with the GCS URI and failure reason
11. THE Models_Module SHALL define Imagen model identifiers with version aliases (e.g., "imagen-3.0-generate-002", "imagen-4.0-generate-preview-05-20")
12. THE Models_Module SHALL define Veo model identifiers with version aliases (e.g., "veo-2.0-generate-001", "veo-3.0-generate-preview")
13. THE Models_Module SHALL define Gemini model identifiers for multimodal generation
14. THE Models_Module SHALL provide model resolution functions that map aliases to full model paths
15. THE Otel_Module SHALL provide optional OpenTelemetry tracing initialization
16. WHEN OpenTelemetry is enabled, THE Otel_Module SHALL configure trace export to Google Cloud Trace

### Requirement 3: MCP Server Framework

**User Story:** As a developer, I want each MCP server to follow a consistent pattern using the rmcp crate, so that servers are maintainable and protocol-compliant.

#### Acceptance Criteria

1. THE MCP_Server SHALL use the rmcp crate for MCP protocol implementation
2. THE MCP_Server SHALL support stdio transport as the default mode
3. THE MCP_Server SHALL support HTTP streamable transport when configured via command-line flag
4. THE MCP_Server SHALL support SSE transport when configured via command-line flag
5. WHEN started with `--transport http`, THE MCP_Server SHALL listen on the configured PORT
6. WHEN started with `--transport sse`, THE MCP_Server SHALL provide Server-Sent Events endpoint
7. THE MCP_Server SHALL register all tools with JSON schema definitions for parameters
8. THE MCP_Server SHALL register all resources with URI templates
9. WHEN a tool is invoked, THE MCP_Server SHALL validate input parameters against the schema
10. IF tool input validation fails, THEN THE MCP_Server SHALL return an MCP error response with validation details
11. WHEN a tool completes successfully, THE MCP_Server SHALL return results as MCP content (text, image, or embedded data)
12. IF a tool execution fails, THEN THE MCP_Server SHALL return an MCP error response with a descriptive message

### Requirement 4: Image Generation Server (adk-rust-mcp-image)

**User Story:** As an AI assistant user, I want to generate images from text prompts using Google's Imagen API, so that I can create visual content programmatically.

#### Acceptance Criteria

1. THE Image_Server SHALL expose a tool named `image_generate` for text-to-image generation
2. THE `image_generate` tool SHALL accept a required `prompt` parameter (string, max 480 characters for Imagen 3, 2000 for Imagen 4)
3. THE `image_generate` tool SHALL accept an optional `negative_prompt` parameter to specify what to avoid
4. THE `image_generate` tool SHALL accept an optional `model` parameter with default "imagen-4.0-generate-preview-05-20"
5. THE `image_generate` tool SHALL accept an optional `aspect_ratio` parameter (1:1, 3:4, 4:3, 9:16, 16:9)
6. THE `image_generate` tool SHALL accept an optional `number_of_images` parameter (1-4, default 1)
7. THE `image_generate` tool SHALL accept an optional `output_file` parameter for local file output
8. THE `image_generate` tool SHALL accept an optional `output_uri` parameter for cloud storage output (supports gs://, s3://, or other storage URIs)
9. WHEN neither output_file nor output_uri is specified, THE `image_generate` tool SHALL return base64-encoded image data
10. WHEN output_file is specified, THE `image_generate` tool SHALL save the image to the local path and return the file path
11. WHEN output_uri is specified, THE `image_generate` tool SHALL upload to the storage backend and return the URI
12. THE Image_Server SHALL expose a resource `image://models` listing available image generation models
13. THE Image_Server SHALL expose a resource `image://segmentation_classes` listing segmentation class options (Google provider specific)
14. WHEN calling Vertex AI Imagen API, THE Image_Server SHALL use ADC for authentication
15. IF the Imagen API returns an error, THEN THE Image_Server SHALL propagate a descriptive error message

### Requirement 5: Video Generation Server (adk-rust-mcp-video)

**User Story:** As an AI assistant user, I want to generate videos from text prompts or images using Google's Veo API, so that I can create video content programmatically.

#### Acceptance Criteria

1. THE Video_Server SHALL expose a tool named `video_generate` for text-to-video generation
2. THE Video_Server SHALL expose a tool named `video_from_image` for image-to-video generation
3. THE `video_generate` tool SHALL accept a required `prompt` parameter (string)
4. THE `video_generate` tool SHALL accept an optional `model` parameter with default "veo-3.0-generate-preview"
5. THE `video_generate` tool SHALL accept an optional `aspect_ratio` parameter (16:9, 9:16)
6. THE `video_generate` tool SHALL accept an optional `duration_seconds` parameter (5-8 seconds depending on model)
7. THE `video_generate` tool SHALL accept a required `output_gcs_uri` parameter (GCS required by Veo API)
8. THE `video_generate` tool SHALL accept an optional `download_local` boolean to also save locally
9. WHEN model is Veo 3.x, THE `video_generate` tool SHALL accept an optional `generate_audio` boolean parameter
10. THE `video_from_image` tool SHALL accept a required `image` parameter (base64 data, local path, or GCS URI)
11. THE `video_from_image` tool SHALL accept a required `prompt` parameter describing the desired video motion
12. THE `video_from_image` tool SHALL accept the same optional parameters as `video_generate`
13. WHEN video generation completes, THE Video_Server SHALL return the GCS URI of the generated video
14. WHEN download_local is true, THE Video_Server SHALL download the video from GCS and return the local path
15. IF the Veo API returns an error, THEN THE Video_Server SHALL propagate a descriptive error message
16. THE Video_Server SHALL poll the long-running operation until completion or timeout

### Requirement 6: Music Generation Server (adk-rust-mcp-music)

**User Story:** As an AI assistant user, I want to generate music from text prompts using Google's Lyria API, so that I can create audio content programmatically.

#### Acceptance Criteria

1. THE Music_Server SHALL expose a tool named `music_generate` for music generation
2. THE `music_generate` tool SHALL accept a required `prompt` parameter describing the desired music
3. THE `music_generate` tool SHALL accept an optional `negative_prompt` parameter to specify what to avoid
4. THE `music_generate` tool SHALL accept an optional `seed` parameter for reproducible generation
5. THE `music_generate` tool SHALL accept an optional `sample_count` parameter (1-4, default 1)
6. THE `music_generate` tool SHALL accept an optional `output_file` parameter for local WAV output
7. THE `music_generate` tool SHALL accept an optional `output_gcs_uri` parameter for GCS output
8. WHEN neither output_file nor output_gcs_uri is specified, THE `music_generate` tool SHALL return base64-encoded WAV data
9. WHEN output_file is specified, THE `music_generate` tool SHALL save the WAV to the local path
10. WHEN output_gcs_uri is specified, THE `music_generate` tool SHALL upload to GCS
11. IF the Lyria API returns an error, THEN THE Music_Server SHALL propagate a descriptive error message

### Requirement 7: Text-to-Speech Server (adk-rust-mcp-speech)

**User Story:** As an AI assistant user, I want to convert text to speech using Google's Chirp3-HD voices, so that I can generate natural-sounding audio narration.

#### Acceptance Criteria

1. THE Speech_Server SHALL expose a tool named `speech_synthesize` for text-to-speech conversion
2. THE Speech_Server SHALL expose a tool named `speech_list_voices` to list available voices
3. THE `speech_synthesize` tool SHALL accept a required `text` parameter (string to synthesize)
4. THE `speech_synthesize` tool SHALL accept an optional `voice` parameter (Chirp3-HD voice name)
5. THE `speech_synthesize` tool SHALL accept an optional `language_code` parameter with default "en-US"
6. THE `speech_synthesize` tool SHALL accept an optional `speaking_rate` parameter (0.25-4.0, default 1.0)
7. THE `speech_synthesize` tool SHALL accept an optional `pitch` parameter (-20.0 to 20.0 semitones, default 0)
8. THE `speech_synthesize` tool SHALL accept an optional `pronunciations` parameter for custom word pronunciations
9. WHEN pronunciations are provided, THE `speech_synthesize` tool SHALL support IPA and X-SAMPA phonetic alphabets
10. THE `speech_synthesize` tool SHALL accept an optional `output_file` parameter for local WAV output
11. WHEN output_file is not specified, THE `speech_synthesize` tool SHALL return base64-encoded WAV data
12. WHEN output_file is specified, THE `speech_synthesize` tool SHALL save the WAV to the local path
13. THE `speech_list_voices` tool SHALL return available Chirp3-HD voice names with language support
14. IF the Cloud TTS API returns an error, THEN THE Speech_Server SHALL propagate a descriptive error message

### Requirement 8: Multimodal Generation Server (adk-rust-mcp-multimodal)

**User Story:** As an AI assistant user, I want to use Gemini's multimodal capabilities for image generation and TTS, so that I can leverage advanced AI features.

#### Acceptance Criteria

1. THE Multimodal_Server SHALL expose a tool named `multimodal_image_generate` for image generation
2. THE Multimodal_Server SHALL expose a tool named `multimodal_speech_synthesize` for text-to-speech
3. THE Multimodal_Server SHALL expose a tool named `multimodal_list_voices` to list available TTS voices
4. THE `multimodal_image_generate` tool SHALL accept a required `prompt` parameter
5. THE `multimodal_image_generate` tool SHALL accept an optional `model` parameter with default Gemini model
6. THE `multimodal_image_generate` tool SHALL accept an optional `output_file` parameter
7. THE `multimodal_speech_synthesize` tool SHALL accept a required `text` parameter
8. THE `multimodal_speech_synthesize` tool SHALL accept an optional `voice` parameter
9. THE `multimodal_speech_synthesize` tool SHALL accept an optional `style` parameter for tone/style control
10. THE `multimodal_speech_synthesize` tool SHALL accept an optional `output_file` parameter
11. WHEN output_file is not specified, THE tools SHALL return base64-encoded data
12. THE Multimodal_Server SHALL expose a resource `multimodal://language_codes` listing supported languages
13. IF the Gemini API returns an error, THEN THE Multimodal_Server SHALL propagate a descriptive error message

### Requirement 9: Audio/Video Processing Server (adk-rust-mcp-avtool)

**User Story:** As an AI assistant user, I want to process audio and video files using FFmpeg, so that I can convert, combine, and manipulate media files.

#### Acceptance Criteria

1. THE AVTool_Server SHALL expose a tool named `ffmpeg_get_media_info` for media file analysis
2. THE AVTool_Server SHALL expose a tool named `ffmpeg_convert_audio_wav_to_mp3` for audio conversion
3. THE AVTool_Server SHALL expose a tool named `ffmpeg_video_to_gif` for GIF creation
4. THE AVTool_Server SHALL expose a tool named `ffmpeg_combine_audio_and_video` for muxing
5. THE AVTool_Server SHALL expose a tool named `ffmpeg_overlay_image_on_video` for image overlay
6. THE AVTool_Server SHALL expose a tool named `ffmpeg_concatenate_media_files` for file concatenation
7. THE AVTool_Server SHALL expose a tool named `ffmpeg_adjust_volume` for audio volume adjustment
8. THE AVTool_Server SHALL expose a tool named `ffmpeg_layer_audio_files` for audio mixing
9. WHEN an input file is a GCS URI, THE AVTool_Server SHALL download it to a temporary location before processing
10. WHEN an output file is a GCS URI, THE AVTool_Server SHALL upload the result after processing
11. THE `ffmpeg_get_media_info` tool SHALL return JSON with duration, codecs, resolution, and stream information
12. THE `ffmpeg_convert_audio_wav_to_mp3` tool SHALL accept bitrate parameter for output quality control
13. THE `ffmpeg_video_to_gif` tool SHALL accept fps, width, and duration parameters
14. THE `ffmpeg_combine_audio_and_video` tool SHALL accept audio and video input paths and output path
15. THE `ffmpeg_overlay_image_on_video` tool SHALL accept position, scale, and duration parameters
16. THE `ffmpeg_concatenate_media_files` tool SHALL accept a list of input files and output path
17. THE `ffmpeg_adjust_volume` tool SHALL accept volume multiplier or dB adjustment
18. THE `ffmpeg_layer_audio_files` tool SHALL accept multiple audio inputs with optional offset and volume
19. IF FFmpeg execution fails, THEN THE AVTool_Server SHALL return the FFmpeg error output
20. IF FFprobe execution fails, THEN THE AVTool_Server SHALL return the FFprobe error output

### Requirement 10: Error Handling and Logging

**User Story:** As a developer, I want consistent error handling and logging across all servers, so that I can diagnose issues effectively.

#### Acceptance Criteria

1. THE Error_Module SHALL define custom error types using thiserror
2. THE Error_Module SHALL categorize errors: ConfigError, ApiError, GcsError, ValidationError, IoError
3. WHEN an error occurs, THE MCP_Server SHALL log the error with context using the tracing crate
4. THE MCP_Server SHALL initialize tracing with env_logger filter support at startup
5. WHEN an API call fails, THE Error_Module SHALL include the API endpoint and response details
6. WHEN a GCS operation fails, THE Error_Module SHALL include the GCS URI and operation type
7. IF a required environment variable is missing, THEN THE Error_Module SHALL return ConfigError with the variable name
8. THE MCP_Server SHALL use anyhow for error propagation with context
9. WHEN returning MCP error responses, THE MCP_Server SHALL include user-friendly error messages
10. THE MCP_Server SHALL support RUST_LOG environment variable for log level configuration

### Requirement 11: Authentication and Security

**User Story:** As a developer, I want secure authentication with Google Cloud services, so that API calls are properly authorized.

#### Acceptance Criteria

1. THE Auth_Module SHALL use Application Default Credentials (ADC) for Google Cloud authentication
2. WHEN ADC is not configured, THE Auth_Module SHALL return a descriptive error with setup instructions
3. THE Auth_Module SHALL support service account key files via GOOGLE_APPLICATION_CREDENTIALS
4. THE Auth_Module SHALL support user credentials from `gcloud auth application-default login`
5. THE Auth_Module SHALL request appropriate OAuth scopes for each API (Vertex AI, Cloud TTS, GCS)
6. THE MCP_Server SHALL NOT log or expose authentication credentials
7. WHEN tokens expire, THE Auth_Module SHALL automatically refresh them

### Requirement 12: Async Runtime and Concurrency

**User Story:** As a developer, I want efficient async processing, so that servers can handle concurrent requests without blocking.

#### Acceptance Criteria

1. THE MCP_Server SHALL use tokio as the async runtime
2. THE MCP_Server SHALL use tokio's multi-threaded runtime for production
3. WHEN making API calls, THE MCP_Server SHALL use async HTTP clients (reqwest or hyper)
4. WHEN performing file I/O, THE MCP_Server SHALL use tokio::fs for non-blocking operations
5. WHEN polling long-running operations, THE MCP_Server SHALL use tokio::time::sleep between polls
6. THE MCP_Server SHALL handle graceful shutdown on SIGTERM and SIGINT signals


### Requirement 13: Multi-Provider Architecture (Phase 2+)

**User Story:** As a developer, I want to use multiple AI providers for media generation, so that I can choose the best provider for each use case or use local inference when cloud APIs are unavailable.

#### Acceptance Criteria

1. THE Provider_Module SHALL define abstract traits for each media type: ImageProvider, VideoProvider, SpeechProvider, MusicProvider
2. THE Provider_Module SHALL allow runtime selection of providers via tool parameters
3. WHEN a `provider` parameter is specified, THE MCP_Server SHALL use that provider for the request
4. WHEN no `provider` parameter is specified, THE MCP_Server SHALL use the default provider from configuration
5. THE Provider_Module SHALL support registering multiple providers for each media type
6. THE Provider_Module SHALL expose a `list_providers` resource showing available providers and their capabilities
7. IF a requested provider is not configured, THEN THE MCP_Server SHALL return an error listing available providers
8. THE Provider_Module SHALL normalize provider-specific responses to common output types (ImageOutput, VideoOutput, AudioOutput)

### Requirement 14: Storage Backend Abstraction (Phase 2+)

**User Story:** As a developer, I want pluggable storage backends, so that I can use GCS, S3, or local filesystem depending on my deployment environment.

#### Acceptance Criteria

1. THE Storage_Module SHALL define a StorageBackend trait with upload, download, exists, get_url, and delete methods
2. THE Storage_Module SHALL implement GcsStorage for Google Cloud Storage
3. THE Storage_Module SHALL implement S3Storage for S3-compatible storage (optional feature)
4. THE Storage_Module SHALL implement LocalStorage for local filesystem storage
5. WHEN an output path starts with `gs://`, THE MCP_Server SHALL use GCS storage
6. WHEN an output path starts with `s3://`, THE MCP_Server SHALL use S3 storage
7. WHEN an output path is a local path, THE MCP_Server SHALL use local filesystem storage
8. THE Storage_Module SHALL support signed URLs for temporary access to stored files

### Requirement 15: Google Cloud Provider (Phase 1)

**User Story:** As a developer, I want Google Cloud as the default provider, so that I can use Vertex AI's generative media APIs.

#### Acceptance Criteria

1. THE Google_Provider SHALL implement ImageProvider using Vertex AI Imagen API
2. THE Google_Provider SHALL implement VideoProvider using Vertex AI Veo API
3. THE Google_Provider SHALL implement SpeechProvider using Cloud TTS Chirp3-HD API
4. THE Google_Provider SHALL implement MusicProvider using Vertex AI Lyria API
5. THE Google_Provider SHALL be enabled by default via the `google` feature flag
6. WHEN the `google` feature is disabled, THE Build_System SHALL not include Google Cloud dependencies

### Requirement 16: OpenAI Provider (Phase 3)

**User Story:** As a developer, I want to use OpenAI's APIs for image and speech generation, so that I can leverage DALL-E and OpenAI TTS.

#### Acceptance Criteria

1. THE OpenAI_Provider SHALL implement ImageProvider using OpenAI DALL-E 3 API
2. THE OpenAI_Provider SHALL implement SpeechProvider using OpenAI TTS API
3. THE OpenAI_Provider SHALL be enabled via the `openai` feature flag
4. THE OpenAI_Provider SHALL load OPENAI_API_KEY from environment variables
5. IF OPENAI_API_KEY is not set when OpenAI provider is requested, THEN THE MCP_Server SHALL return a configuration error

### Requirement 17: Local Inference Provider (Phase 4)

**User Story:** As a developer, I want to run inference locally using mistral.rs, so that I can generate media without cloud API dependencies.

#### Acceptance Criteria

1. THE Local_Provider SHALL implement ImageProvider using mistral.rs diffusion models (FLUX)
2. THE Local_Provider SHALL implement SpeechProvider using mistral.rs speech models (Dia)
3. THE Local_Provider SHALL be enabled via the `local` feature flag
4. THE Local_Provider SHALL support device selection (CPU, CUDA, Metal) via configuration
5. THE Local_Provider SHALL cache downloaded models in a configurable directory
6. WHEN a model is not cached, THE Local_Provider SHALL download it from HuggingFace Hub
7. THE Local_Provider SHALL support model preloading at startup for faster first inference

### Requirement 18: Ollama Provider (Phase 4)

**User Story:** As a developer, I want to use Ollama for local inference, so that I can leverage models already running on my system.

#### Acceptance Criteria

1. THE Ollama_Provider SHALL implement ImageProvider for Ollama models with image generation capability
2. THE Ollama_Provider SHALL be enabled via the `ollama` feature flag
3. THE Ollama_Provider SHALL connect to Ollama at the configured host (default: localhost:11434)
4. THE Ollama_Provider SHALL query available models from the Ollama API
5. IF Ollama is not running, THEN THE MCP_Server SHALL return a connection error

### Requirement 19: Replicate Provider (Phase 5)

**User Story:** As a developer, I want to use Replicate's API for diverse model access, so that I can use models like Stable Diffusion XL and other community models.

#### Acceptance Criteria

1. THE Replicate_Provider SHALL implement ImageProvider using Replicate predictions API
2. THE Replicate_Provider SHALL implement VideoProvider using Replicate predictions API
3. THE Replicate_Provider SHALL be enabled via the `replicate` feature flag
4. THE Replicate_Provider SHALL load REPLICATE_API_TOKEN from environment variables
5. THE Replicate_Provider SHALL poll prediction status until completion
6. IF REPLICATE_API_TOKEN is not set when Replicate provider is requested, THEN THE MCP_Server SHALL return a configuration error

### Requirement 20: Feature Flags and Conditional Compilation

**User Story:** As a developer, I want to compile only the providers I need, so that binary size is minimized and unnecessary dependencies are excluded.

#### Acceptance Criteria

1. THE Build_System SHALL use Cargo feature flags to enable/disable providers
2. THE `google` feature SHALL be enabled by default
3. THE `openai` feature SHALL enable OpenAI provider dependencies
4. THE `local` feature SHALL enable mistral.rs dependencies
5. THE `ollama` feature SHALL enable Ollama client dependencies
6. THE `replicate` feature SHALL enable Replicate API dependencies
7. THE `s3` feature SHALL enable S3 storage backend dependencies
8. THE `all-providers` feature SHALL enable all provider features
9. WHEN a feature is disabled, THE Build_System SHALL exclude related code via conditional compilation
