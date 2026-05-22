# adk-rust-mcp-video

MCP server for video generation. Part of the [ADK Rust MCP toolkit](https://github.com/zavora-ai/adk-rust-mcp-toolkit).

## Overview

Video generation server supporting text-to-video, image-to-video, video interpolation, and video extension via Google's Veo API. Uses long-running operations with automatic polling and exponential backoff.

**Currently implemented:** Google Vertex AI / Gemini API (Veo 3.1, Veo 3, Veo 2)

## Features

- **Text-to-Video** — Generate videos from text prompts with Veo 3.1
- **Image-to-Video** — Animate images into videos
- **Video Interpolation** — Generate video between two keyframes (first + last frame)
- **Video Extension** — Extend existing videos with new content
- **Audio Generation** — Generate audio with video (Veo 3.x+)
- **Local Download** — Optionally download generated videos locally
- **Dual API** — Works with Gemini API key or Vertex AI ADC

## Installation

```bash
cargo install adk-rust-mcp-video
```

## Configuration

```bash
# Option 1: Gemini API (recommended for getting started)
export GEMINI_API_KEY=your-api-key

# Option 2: Vertex AI (for production/enterprise)
export PROJECT_ID=your-gcp-project
export LOCATION=us-central1
export GCS_BUCKET=your-bucket  # Required for Vertex AI video output
```

## Tools

### video_generate

Generate videos from text prompts.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | — | Text describing the video |
| `output_gcs_uri` | string | Yes | — | GCS URI for output (e.g., `gs://bucket/path`) |
| `model` | string | No | `veo-3.1-generate-preview` | Model ID or alias |
| `aspect_ratio` | string | No | `16:9` | `16:9` or `9:16` |
| `duration_seconds` | int | No | 8 | Duration: 4, 6, or 8 seconds |
| `generate_audio` | bool | No | false | Generate audio (Veo 3.x+ only) |
| `seed` | int | No | — | Random seed |
| `download_local` | bool | No | false | Download to local filesystem |
| `local_path` | string | No | — | Local path for download |

### video_from_image

Generate video from an image (or interpolate between two images).

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `image` | string | Yes | — | Source image (base64, local path, or GCS URI) |
| `prompt` | string | Yes | — | Desired video motion description |
| `output_gcs_uri` | string | Yes | — | GCS URI for output |
| `last_frame_image` | string | No | — | Last frame for interpolation mode |
| `model` | string | No | `veo-3.1-generate-preview` | Model ID |
| `aspect_ratio` | string | No | `16:9` | `16:9` or `9:16` |
| `duration_seconds` | int | No | — | Duration: 5-8 seconds |
| `seed` | int | No | — | Random seed |
| `download_local` | bool | No | false | Download locally |
| `local_path` | string | No | — | Local path for download |

### video_extend

Extend an existing video with new content.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `video_input` | string | Yes | — | GCS URI of video to extend |
| `prompt` | string | Yes | — | Desired continuation description |
| `output_gcs_uri` | string | Yes | — | GCS URI for output |
| `model` | string | No | `veo-3.1-generate-preview` | Model ID |
| `duration_seconds` | int | No | — | Duration: 5-8 seconds |
| `seed` | int | No | — | Random seed |
| `download_local` | bool | No | false | Download locally |
| `local_path` | string | No | — | Local path for download |

## Usage Examples

```bash
# Stdio (default) — for Claude Desktop, Kiro
adk-rust-mcp-video

# HTTP — for web apps, ADK agents
adk-rust-mcp-video --transport http --port 8080

# SSE — for streaming applications
adk-rust-mcp-video --transport sse --port 8080
```

### Generate a video

```
prompt: "A drone shot flying over a misty mountain range at sunrise"
output_gcs_uri: "gs://my-bucket/videos/mountains.mp4"
generate_audio: true
duration_seconds: 8
```

### Animate an image

```
image: "landscape.png"
prompt: "Camera slowly pans right, clouds drift across the sky"
output_gcs_uri: "gs://my-bucket/videos/animated.mp4"
```

## Supported Models

| Model | ID | Audio | Durations |
|-------|-----|-------|-----------|
| Veo 3.1 | `veo-3.1-generate-preview` | Yes | 4, 6, 8s |
| Veo 3.1 Fast | `veo-3.1-fast-generate-preview` | Yes | 4, 6, 8s |
| Veo 3.1 Lite | `veo-3.1-lite-generate-preview` | Yes | 4, 6, 8s |

## Output Specs

| Format | Resolution | Duration |
|--------|-----------|----------|
| MP4 (H.264) | Up to 1080p | 4-8 seconds |

## License

Apache-2.0
