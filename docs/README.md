# ADK Rust MCP Toolkit

A collection of Model Context Protocol (MCP) servers for generative media, built in Rust. Designed to be provider-agnostic with support for multiple AI backends.

## Overview

This workspace provides MCP servers for:

| Server | Description | Status |
|--------|-------------|--------|
| `adk-rust-mcp-image` | Image generation and upscaling | ✅ Complete |
| `adk-rust-mcp-video` | Video generation (text-to-video, image-to-video) | ✅ Complete |
| `adk-rust-mcp-music` | Music generation from text prompts | ✅ Complete |
| `adk-rust-mcp-speech` | Text-to-speech synthesis | ✅ Complete |
| `adk-rust-mcp-multimodal` | Multimodal generation (image, TTS) | ✅ Complete |
| `adk-rust-mcp-avtool` | Audio/video processing with FFmpeg | ✅ Complete |

### Supported Providers

Currently implemented:
- **Google Cloud** - Vertex AI (Imagen, Veo, Lyria), Cloud TTS, Gemini

Planned:
- AWS Bedrock
- Azure OpenAI
- Local/self-hosted models

## Quick Start

### Prerequisites

- Rust 2024 edition
- Google Cloud SDK with authenticated credentials
- Vertex AI API enabled in your GCP project
- FFmpeg installed (for adk-rust-mcp-avtool)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd adk-rust-mcp

# Build all servers
cargo build --release
```

### Configuration

Create a `.env` file in the workspace root:

```bash
# Required
PROJECT_ID=your-gcp-project-id

# Optional
LOCATION=us-central1          # Default: us-central1
GCS_BUCKET=your-bucket-name   # For cloud storage output
PORT=8080                     # Default: 8080
RUST_LOG=info                 # Logging level
```

## Environment Variables

### Core Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PROJECT_ID` | Yes | - | Google Cloud project ID |
| `LOCATION` | No | `us-central1` | GCP region for Vertex AI |
| `GCS_BUCKET` | No | - | GCS bucket for output storage |
| `PORT` | No | `8080` | HTTP/SSE server port |
| `RUST_LOG` | No | `info` | Logging level (trace, debug, info, warn, error) |

### Authentication

The servers use Google Cloud Application Default Credentials (ADC). Set up authentication using one of:

```bash
# Option 1: User credentials (development)
gcloud auth application-default login

# Option 2: Service account (production)
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account-key.json
```

## Transport Options

All servers support three transport modes:

### Stdio (Default)

Standard input/output transport for local subprocess communication:

```bash
./target/release/adk-rust-mcp-image
```

### HTTP Streamable

HTTP transport for web-based clients:

```bash
./target/release/adk-rust-mcp-image --transport http --port 8080
```

### SSE (Server-Sent Events)

Real-time streaming over HTTP:

```bash
./target/release/adk-rust-mcp-image --transport sse --port 8080
```

## Running Servers

### Image Generation Server

```bash
# Stdio transport (default)
./target/release/adk-rust-mcp-image

# HTTP transport
./target/release/adk-rust-mcp-image --transport http --port 8080
```

**Tools:**
- `image_generate` - Generate images from text prompts

**Resources:**
- `image://models` - Available image generation models
- `image://segmentation_classes` - Segmentation classes (Google provider)
- `image://providers` - Available providers

### Video Generation Server

```bash
./target/release/adk-rust-mcp-video
```

**Tools:**
- `video_generate` - Generate videos from text prompts
- `video_from_image` - Generate videos from images

**Resources:**
- `video://models` - Available video generation models
- `video://providers` - Available providers

### Music Generation Server

```bash
./target/release/adk-rust-mcp-music
```

**Tools:**
- `music_generate` - Generate music from text prompts

### Speech Synthesis Server

```bash
./target/release/adk-rust-mcp-speech
```

**Tools:**
- `speech_synthesize` - Convert text to speech
- `speech_list_voices` - List available voices

### Multimodal Generation Server

```bash
./target/release/adk-rust-mcp-multimodal
```

**Tools:**
- `multimodal_image_generate` - Generate images using Gemini
- `multimodal_speech_synthesize` - Text-to-speech using Gemini
- `multimodal_list_voices` - List available Gemini TTS voices

**Resources:**
- `multimodal://language_codes` - Supported language codes

### Audio/Video Processing Server

```bash
./target/release/adk-rust-mcp-avtool
```

**Tools:**
- `ffmpeg_get_media_info` - Get media file information
- `ffmpeg_convert_audio_wav_to_mp3` - Convert WAV to MP3
- `ffmpeg_video_to_gif` - Convert video to GIF
- `ffmpeg_combine_audio_and_video` - Combine audio and video tracks
- `ffmpeg_overlay_image_on_video` - Overlay image on video
- `ffmpeg_concatenate_media_files` - Concatenate media files
- `ffmpeg_adjust_volume` - Adjust audio volume
- `ffmpeg_layer_audio_files` - Layer/mix multiple audio files

## MCP Client Configuration

### Claude Desktop

Add to your Claude Desktop configuration (`~/.config/claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "image-gen": {
      "command": "/path/to/adk-rust-mcp-image",
      "args": []
    },
    "video-gen": {
      "command": "/path/to/adk-rust-mcp-video",
      "args": []
    },
    "music-gen": {
      "command": "/path/to/adk-rust-mcp-music",
      "args": []
    },
    "speech": {
      "command": "/path/to/adk-rust-mcp-speech",
      "args": []
    },
    "multimodal": {
      "command": "/path/to/adk-rust-mcp-multimodal",
      "args": []
    },
    "avtool": {
      "command": "/path/to/adk-rust-mcp-avtool",
      "args": []
    }
  }
}
```

### Kiro

Add to your Kiro MCP configuration (`.kiro/settings/mcp.json`):

```json
{
  "mcpServers": {
    "image-gen": {
      "command": "/path/to/adk-rust-mcp-image",
      "args": [],
      "env": {
        "PROJECT_ID": "your-project-id",
        "LOCATION": "us-central1"
      }
    }
  }
}
```

## Testing

```bash
# Run all tests
cargo test --workspace

# Run property-based tests with more iterations
PROPTEST_CASES=1000 cargo test --workspace

# Run specific crate tests
cargo test -p adk-rust-mcp-common
cargo test -p adk-rust-mcp-image

# Run workspace integration tests
cargo test -p workspace-integration-tests
```

## Documentation

### Server Guides
- [Image Server](./servers/image.md) - Image generation with Imagen
- [Video Server](./servers/video.md) - Video generation with Veo
- [Music Server](./servers/music.md) - Music generation with Lyria
- [Speech Server](./servers/speech.md) - Text-to-speech with Chirp3-HD
- [Multimodal Server](./servers/multimodal.md) - Multimodal generation with Gemini
- [AVTool Server](./servers/avtool.md) - Audio/video processing with FFmpeg

### Reference
- [API Reference](./api/README.md) - Tool and resource schemas
- [Configuration](./configuration.md) - Environment variables and settings
- [Development](./development.md) - Contributing and development guide

## License

See [LICENSE](../LICENSE) for details.
