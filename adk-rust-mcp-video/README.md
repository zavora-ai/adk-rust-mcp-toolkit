# adk-rust-mcp-video

MCP server for video generation. Part of the ADK Rust MCP toolkit.

## Overview

Provider-agnostic video generation server designed to support multiple AI backends.

```
┌─────────────────────────────────────────────────┐
│              adk-rust-mcp-video                  │
├─────────────────────────────────────────────────┤
│              Provider Abstraction                │
├──────────┬──────────┬──────────┬────────────────┤
│   Veo    │  Sora    │ Runway   │   Local        │
│ (Google) │ (OpenAI) │   Gen    │   Models       │
└──────────┴──────────┴──────────┴────────────────┘
```

**Currently implemented:** Google Vertex AI Veo

**Planned:** OpenAI Sora, Runway, Local models

## Features

- **Text-to-Video** - Generate videos from text prompts
- **Image-to-Video** - Animate images into videos
- **Video Interpolation** - Generate video between two keyframes
- **Video Extension** - Extend existing videos with new content
- **Audio Generation** - Generate audio with video (Veo 3.x)
- **Local Download** - Optionally download generated videos locally

## Installation

```bash
cargo install adk-rust-mcp-video
```

Or build from source:

```bash
cargo build --release --package adk-rust-mcp-video
```

## Configuration

```bash
export PROJECT_ID=your-gcp-project
export LOCATION=us-central1
export GCS_BUCKET=your-bucket  # Required for video output
```

**Note:** Video generation requires cloud storage output.

## Usage

### Running the Server

```bash
# Stdio transport (for Claude Desktop, Kiro)
adk-rust-mcp-video

# HTTP transport (for web clients, ADK agents)
adk-rust-mcp-video --transport http --port 8080
```

### MCP Client Configuration

**Kiro** (`.kiro/settings/mcp.json`):

```json
{
  "mcpServers": {
    "video": {
      "command": "/path/to/adk-rust-mcp-video",
      "args": ["--transport", "stdio"],
      "cwd": "/path/to/workspace",
      "env": {
        "PROJECT_ID": "your-project",
        "GCS_BUCKET": "your-bucket"
      }
    }
  }
}
```

## Tools

### video_generate

Generate videos from text prompts.

| Parameter | Type | Required | Default |
|-----------|------|----------|---------|
| `prompt` | string | Yes | - |
| `output_gcs_uri` | string | Yes | - |
| `model` | string | No | `veo-3` |
| `aspect_ratio` | string | No | `16:9` |
| `duration_seconds` | int | No | 8 |
| `generate_audio` | bool | No | false |
| `download_local` | bool | No | false |
| `local_path` | string | No | - |

### video_from_image

Generate video from an image.

| Parameter | Type | Required | Default |
|-----------|------|----------|---------|
| `image` | string | Yes | - |
| `prompt` | string | Yes | - |
| `output_gcs_uri` | string | Yes | - |
| `last_frame_image` | string | No | - |

### video_extend

Extend an existing video.

| Parameter | Type | Required | Default |
|-----------|------|----------|---------|
| `video_input` | string | Yes | - |
| `prompt` | string | Yes | - |
| `output_gcs_uri` | string | Yes | - |

## Resources

- `video://models` - List available models
- `video://providers` - List providers

## Supported Models (Google Veo)

| Model | Aliases | Audio | Durations |
|-------|---------|-------|-----------|
| `veo-3.0-generate-preview` | `veo-3` | Yes | 4, 6, 8s |
| `veo-2.0-generate-001` | `veo-2` | No | 4, 6, 8s |

## License

Apache-2.0
