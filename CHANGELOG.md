# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-24

### Added

#### MCP Servers
- **adk-rust-mcp-image**: Image generation server using Vertex AI Imagen
  - `image_generate` tool for text-to-image generation
  - `image_upscale` tool for image upscaling (x2, x4)
  - Support for multiple aspect ratios (1:1, 3:4, 4:3, 9:16, 16:9)
  - Batch generation (1-4 images per request)
  - Output to base64, local files, or GCS
  - Resources: `image://models`, `image://providers`, `image://segmentation_classes`

- **adk-rust-mcp-video**: Video generation server using Vertex AI Veo
  - `video_generate` tool for text-to-video generation
  - `video_from_image` tool for image-to-video (single image or interpolation)
  - `video_extend` tool for extending existing videos
  - Support for Veo 2.x and 3.x models
  - Audio generation support (Veo 3.x)
  - LRO polling with exponential backoff
  - Resources: `video://models`, `video://providers`

- **adk-rust-mcp-music**: Music generation server using Vertex AI Lyria
  - `music_generate` tool for text-to-music generation
  - Batch generation (1-4 samples)
  - Negative prompts for refined control
  - Output to base64, local files, or GCS

- **adk-rust-mcp-speech**: Text-to-speech server using Cloud TTS Chirp3-HD
  - `speech_synthesize` tool for text-to-speech
  - `speech_list_voices` tool for listing available voices
  - Adjustable speaking rate and pitch
  - Custom pronunciations using IPA or X-SAMPA alphabets
  - SSML support

- **adk-rust-mcp-multimodal**: Multimodal generation server using Gemini
  - `multimodal_image_generate` tool for image generation
  - `multimodal_speech_synthesize` tool for TTS with style control
  - `multimodal_list_voices` tool for listing voices
  - Resources: `multimodal://language_codes`, `multimodal://voices`

- **adk-rust-mcp-avtool**: Audio/video processing server using FFmpeg
  - `ffmpeg_get_media_info` - Get media file information
  - `ffmpeg_convert_audio_wav_to_mp3` - Convert WAV to MP3
  - `ffmpeg_video_to_gif` - Convert video to GIF
  - `ffmpeg_combine_audio_and_video` - Combine audio and video
  - `ffmpeg_overlay_image_on_video` - Overlay image on video
  - `ffmpeg_concatenate_media_files` - Concatenate media files
  - `ffmpeg_adjust_volume` - Adjust audio volume
  - `ffmpeg_layer_audio_files` - Layer/mix audio files
  - Support for local files and GCS URIs

#### Common Library (adk-rust-mcp-common)
- Shared configuration management
- Google Cloud authentication provider
- GCS client for cloud storage operations
- Error types and handling
- MCP server builder with transport abstraction
- Transport options: stdio, HTTP, SSE
- OpenTelemetry tracing support (optional feature)

#### Documentation
- Comprehensive server documentation in `docs/servers/`
- API reference documentation in `docs/api/`
- Configuration guide
- Development guide
- Feature parity analysis

#### Development Infrastructure
- Kiro hooks for automated documentation updates
  - `update-api-docs` - Updates API docs on handler changes
  - `update-server-docs` - Updates server docs on server changes
  - `update-readme-new-server` - Creates docs for new servers
  - `audit-documentation` - Manual audit hook for documentation completeness
- Steering documents for development patterns
  - `rmcp-server-patterns.md` - RMCP implementation patterns
  - `documentation-maintenance.md` - Documentation maintenance guide
- Property-based testing with proptest
- Integration tests for all servers
- Workspace-level integration tests

#### Project Setup
- Apache 2.0 license (Zavora Technologies Ltd)
- Contributing guidelines
- Project README with quick start guide
- Provider-agnostic architecture design

### Technical Details

#### Dependencies
- rmcp v0.13 for MCP protocol implementation
- tokio for async runtime
- reqwest for HTTP client
- serde/serde_json for serialization
- schemars for JSON Schema generation
- clap for CLI argument parsing
- tracing for logging

#### Supported Providers
- Google Cloud (Vertex AI, Cloud TTS, Gemini) - Implemented
- AWS Bedrock - Planned
- Azure OpenAI - Planned
- Local/self-hosted models - Planned

[0.1.0]: https://github.com/zavora-ai/adk-rust-mcp-toolkit/releases/tag/v0.1.0
