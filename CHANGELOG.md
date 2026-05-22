# Changelog

## [0.5.0] - 2026-05-22

### Added
- **Dual API Support** — Gemini API (API key) and Vertex AI (ADC) with auto-detection
- **Lyria RealTime** — Interactive music streaming via WebSocket (start/steer/stop)
- **adk-rust-mcp-composer** — GIF, short-form video, meme, presentation, podcast generation
- **adk-rust-mcp-education** — Whiteboard, flashcard, story, quiz, explainer tools
- **adk-rust-mcp-diagrams** — Mermaid/SVG/PlantUML diagram generation from natural language
- **adk-rust-mcp-artist** — Art creation with 8 styles, style transfer, sketch-to-art, variations
- **adk-rust-mcp-graphics** — Image editing, object removal, background swap, resize, enhance
- **New models** — Veo 3.1 (generate/fast/lite), Lyria 3 Pro/Clip, Gemini 3.1 Flash Image (Nano Banana 2)
- **30 TTS voices** — Full voice set with 70+ language support
- **Industry use cases** — Documentation for 10 key sectors

### Fixed
- Tracing output directed to stderr for stdio MCP transport
- Correct WebSocket protocol for Lyria RealTime (BidiGenerateMusic)
- Skip `storageUri` for Gemini API video requests
- Proper base64 audio decoding from WebSocket binary frames

### Changed
- Common crate uses local path dependency for development
- All README images use absolute GitHub raw URLs (crates.io compatible)

## [0.3.0] - 2026-04-28

### Added
- Initial release with 6 MCP servers
- Image generation (Imagen 3/4) and upscaling
- Video generation (Veo 2/3) with LRO polling
- Music generation (Lyria 1.0)
- Speech synthesis (Chirp3-HD)
- Multimodal generation (Gemini 2.0 Flash)
- FFmpeg audio/video processing (8 tools)
- Stdio, HTTP, and SSE transport support
- GCS upload/download
- OpenTelemetry tracing (optional)

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-24

### Changed

- **Breaking**: Upgraded rmcp from 0.13 to 0.14
  - Updated deprecated type names: `PaginatedRequestParam` → `PaginatedRequestParams`
  - Updated deprecated type names: `CallToolRequestParam` → `CallToolRequestParams`
  - Updated deprecated type names: `ReadResourceRequestParam` → `ReadResourceRequestParams`

- **Examples**: Updated all ADK agent examples to use HTTP Streamable transport
  - `image-agent`, `video-agent`, `music-agent`, `speech-agent` now connect via HTTP
  - `media-pipeline` and `creative-studio` connect to multiple servers via HTTP
  - Removed stdio transport from examples (servers run separately)
  - Added configurable MCP endpoints via environment variables

### Added

- HTTP Streamable transport support for all MCP servers
- Environment variable configuration for MCP endpoints in examples
- Updated documentation for HTTP transport usage

### Fixed

- Fixed rmcp 0.14 compatibility issues across all server implementations

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

#### Examples
- **image-agent**: ADK agent for image generation
- **video-agent**: ADK agent for video generation
- **music-agent**: ADK agent for music composition
- **speech-agent**: ADK agent for text-to-speech
- **media-pipeline**: Multi-tool orchestration agent
- **creative-studio**: Full creative suite agent

#### Documentation
- Comprehensive server documentation in `docs/servers/`
- API reference documentation in `docs/api/`
- Configuration guide
- Development guide
- Feature parity analysis

#### Development Infrastructure
- Kiro hooks for automated documentation updates
- Steering documents for development patterns
- Property-based testing with proptest
- Integration tests for all servers
- Workspace-level integration tests

### Technical Details

#### Dependencies
- rmcp v0.14 for MCP protocol implementation
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

[0.2.0]: https://github.com/anthropics/adk-rust-mcp/releases/tag/v0.2.0
[0.1.0]: https://github.com/anthropics/adk-rust-mcp/releases/tag/v0.1.0
