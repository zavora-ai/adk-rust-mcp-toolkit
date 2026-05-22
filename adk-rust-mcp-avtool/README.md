# adk-rust-mcp-avtool

MCP server for audio/video processing with FFmpeg. Part of the [ADK Rust MCP toolkit](https://github.com/zavora-ai/adk-rust-mcp-toolkit).

## Overview

Media processing server providing FFmpeg-powered tools for post-processing generated content. Supports format conversion, audio mixing, video manipulation, and cloud storage I/O.

## Features

- **Media Info** — Get duration, format, codec, and stream information
- **Audio Conversion** — WAV to MP3 with configurable bitrate
- **Video to GIF** — Animated GIFs with palette optimization
- **Audio/Video Combine** — Merge separate audio and video tracks
- **Image Overlay** — Add images/watermarks to videos with positioning
- **Concatenation** — Join multiple media files sequentially
- **Volume Control** — Adjust audio levels (multiplier or dB)
- **Audio Layering** — Mix multiple audio tracks with offset and volume control
- **Cloud Storage** — Read from and write to GCS URIs

## Prerequisites

FFmpeg must be installed:

```bash
# macOS
brew install ffmpeg

# Ubuntu/Debian
sudo apt install ffmpeg
```

## Installation

```bash
cargo install adk-rust-mcp-avtool
```

## Configuration

```bash
# Optional: for GCS URI support
export GEMINI_API_KEY=your-api-key
# or
export PROJECT_ID=your-gcp-project
export LOCATION=us-central1
```

## Tools

### ffmpeg_get_media_info

Get information about a media file (duration, format, streams, codecs).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input` | string | Yes | Input file path or GCS URI |

### ffmpeg_convert_audio_wav_to_mp3

Convert a WAV audio file to MP3 format.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `input` | string | Yes | — | Input WAV file path or GCS URI |
| `output` | string | Yes | — | Output MP3 file path or GCS URI |
| `bitrate` | string | No | `192k` | Audio bitrate (128k, 192k, 320k) |

### ffmpeg_video_to_gif

Convert a video file to animated GIF.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `input` | string | Yes | — | Input video file |
| `output` | string | Yes | — | Output GIF file |
| `fps` | int | No | 10 | Frames per second |
| `width` | int | No | — | Output width (auto height) |
| `start_time` | float | No | — | Start time in seconds |
| `duration` | float | No | — | Duration in seconds |

### ffmpeg_combine_audio_and_video

Combine separate audio and video files into a single file.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `video_input` | string | Yes | Input video file |
| `audio_input` | string | Yes | Input audio file |
| `output` | string | Yes | Output file |

### ffmpeg_overlay_image_on_video

Overlay an image on a video at a specified position.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `video_input` | string | Yes | — | Input video file |
| `image_input` | string | Yes | — | Image to overlay |
| `output` | string | Yes | — | Output file |
| `x` | int | No | 0 | X position from left |
| `y` | int | No | 0 | Y position from top |
| `scale` | float | No | — | Scale factor (e.g., 0.5) |
| `start_time` | float | No | — | When overlay appears |
| `duration` | float | No | — | How long overlay shows |

### ffmpeg_concatenate_media_files

Concatenate multiple media files into a single file.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `inputs` | array | Yes | List of input file paths |
| `output` | string | Yes | Output file |

### ffmpeg_adjust_volume

Adjust the volume of an audio file.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input` | string | Yes | Input audio file |
| `output` | string | Yes | Output audio file |
| `volume` | string | Yes | Volume: `"0.5"`, `"2.0"`, `"-3dB"`, `"+6dB"` |

### ffmpeg_layer_audio_files

Layer/mix multiple audio files with optional offset and volume control.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `inputs` | array | Yes | Audio layers (see below) |
| `output` | string | Yes | Output file |

**Audio layer format:**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `path` | string | — | Input audio file path |
| `offset_seconds` | float | 0.0 | Offset from start |
| `volume` | float | 1.0 | Volume multiplier |

## Usage Examples

```bash
# Stdio (default) — for Claude Desktop, Kiro
adk-rust-mcp-avtool

# HTTP — for web apps, ADK agents
adk-rust-mcp-avtool --transport http --port 8080

# SSE — for streaming applications
adk-rust-mcp-avtool --transport sse --port 8080
```

### Cloud storage support

All tools support GCS URIs for input and output:

```json
{
  "input": "gs://bucket/input.wav",
  "output": "gs://bucket/output.mp3"
}
```

## Supported Formats

| Type | Formats |
|------|---------|
| Audio | WAV, MP3, OGG, FLAC, AAC |
| Video | MP4, WebM, MKV, AVI, MOV |
| Image | PNG, JPEG, GIF |

## License

Apache-2.0
