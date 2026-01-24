# ADK Rust MCP Toolkit Documentation

A collection of Model Context Protocol (MCP) servers for generative media, built in Rust. Designed to be provider-agnostic with support for multiple AI backends.

## Overview

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

- Rust 2024 edition (1.85+)
- Google Cloud SDK with authenticated credentials
- Vertex AI API enabled in your GCP project
- FFmpeg installed (for adk-rust-mcp-avtool)

### Installation

```bash
git clone https://github.com/anthropics/adk-rust-mcp
cd adk-rust-mcp
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

## Transport Options

All servers support three transport modes:

### Stdio (Default)

Standard input/output transport for local subprocess communication. Used by Claude Desktop, Kiro, and other MCP clients that spawn servers as child processes.

```bash
./target/release/adk-rust-mcp-image
```

### HTTP Streamable (Recommended for Remote Clients)

HTTP transport using the MCP Streamable HTTP protocol. Recommended for ADK agents, web applications, and remote clients.

```bash
./target/release/adk-rust-mcp-image --transport http --port 8080
```

The server exposes the MCP endpoint at `/mcp` (e.g., `http://localhost:8080/mcp`).

### SSE (Server-Sent Events)

Real-time streaming over HTTP using Server-Sent Events.

```bash
./target/release/adk-rust-mcp-image --transport sse --port 8080
```

## Running Servers

### Image Generation Server

```bash
# Stdio (default)
./target/release/adk-rust-mcp-image

# HTTP
./target/release/adk-rust-mcp-image --transport http --port 8080
```

**Tools:** `image_generate`, `image_upscale`

**Resources:** `image://models`, `image://providers`, `image://segmentation_classes`

### Video Generation Server

```bash
./target/release/adk-rust-mcp-video --transport http --port 8081
```

**Tools:** `video_generate`, `video_from_image`, `video_extend`

**Resources:** `video://models`, `video://providers`

### Music Generation Server

```bash
./target/release/adk-rust-mcp-music --transport http --port 8082
```

**Tools:** `music_generate`

### Speech Synthesis Server

```bash
./target/release/adk-rust-mcp-speech --transport http --port 8083
```

**Tools:** `speech_synthesize`, `speech_list_voices`

### Multimodal Generation Server

```bash
./target/release/adk-rust-mcp-multimodal --transport http --port 8084
```

**Tools:** `multimodal_image_generate`, `multimodal_speech_synthesize`, `multimodal_list_voices`

**Resources:** `multimodal://language_codes`, `multimodal://voices`

### Audio/Video Processing Server

```bash
./target/release/adk-rust-mcp-avtool --transport http --port 8085
```

**Tools:** `ffmpeg_get_media_info`, `ffmpeg_convert_audio_wav_to_mp3`, `ffmpeg_video_to_gif`, `ffmpeg_combine_audio_and_video`, `ffmpeg_overlay_image_on_video`, `ffmpeg_concatenate_media_files`, `ffmpeg_adjust_volume`, `ffmpeg_layer_audio_files`

## MCP Client Configuration

### Claude Desktop

Add to `~/.config/claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "image-gen": {
      "command": "/path/to/adk-rust-mcp-image",
      "env": {
        "PROJECT_ID": "your-project-id",
        "LOCATION": "us-central1"
      }
    },
    "video-gen": {
      "command": "/path/to/adk-rust-mcp-video",
      "env": {
        "PROJECT_ID": "your-project-id"
      }
    }
  }
}
```

### Kiro

Add to `.kiro/settings/mcp.json`:

```json
{
  "mcpServers": {
    "image-gen": {
      "command": "/path/to/adk-rust-mcp-image",
      "env": {
        "PROJECT_ID": "your-project-id",
        "LOCATION": "us-central1"
      }
    }
  }
}
```

### ADK-Rust Agents

For ADK-Rust agents, use HTTP transport:

```rust
use adk_tool::McpHttpClientBuilder;
use std::time::Duration;

// Connect to MCP server running on HTTP
let toolset = McpHttpClientBuilder::new("http://localhost:8080/mcp")
    .timeout(Duration::from_secs(60))
    .connect()
    .await?;

// Discover and use tools
let tools = toolset.tools(ctx).await?;
```

See the [examples](../examples/README.md) directory for complete agent implementations.

## Authentication

The servers use Google Cloud Application Default Credentials (ADC):

```bash
# Option 1: User credentials (development)
gcloud auth application-default login

# Option 2: Service account (production)
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account-key.json
```

Required IAM roles:
- `roles/aiplatform.user` - For Vertex AI API access
- `roles/storage.objectAdmin` - For GCS read/write (if using GCS output)

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p adk-rust-mcp-image

# Run integration tests (requires GCP credentials)
cargo test --test integration_test

# Skip integration tests
SKIP_INTEGRATION_TESTS=1 cargo test
```

## Documentation

### Server Guides
- [Image Server](./servers/image.md)
- [Video Server](./servers/video.md)
- [Music Server](./servers/music.md)
- [Speech Server](./servers/speech.md)
- [Multimodal Server](./servers/multimodal.md)
- [AVTool Server](./servers/avtool.md)

### Reference
- [API Reference](./api/README.md)
- [Configuration](./configuration.md)
- [Development](./development.md)

## License

See [LICENSE](../LICENSE) for details.
