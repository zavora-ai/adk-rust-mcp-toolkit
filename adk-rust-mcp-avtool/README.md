# adk-rust-mcp-avtool

MCP server for audio/video processing. Part of the ADK Rust MCP toolkit.

## Overview

Audio/video processing server using FFmpeg. Provides media manipulation tools for post-processing generated content.

## Features

- **Media Info** - Get duration, format, codec information
- **Audio Conversion** - Convert WAV to MP3
- **Video to GIF** - Create animated GIFs from videos
- **Audio/Video Combine** - Merge separate tracks
- **Image Overlay** - Add images/watermarks to videos
- **Concatenation** - Join multiple media files
- **Volume Control** - Adjust audio levels
- **Audio Layering** - Mix multiple audio tracks
- **Cloud Storage** - Read from and write to GCS

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
export PROJECT_ID=your-gcp-project  # optional, for GCS
```

## Usage

```bash
# Stdio transport
adk-rust-mcp-avtool

# HTTP transport
adk-rust-mcp-avtool --transport http --port 8080
```

## Tools

### ffmpeg_get_media_info

Get information about a media file.

| Parameter | Type | Required |
|-----------|------|----------|
| `input` | string | Yes |

### ffmpeg_convert_audio_wav_to_mp3

| Parameter | Type | Required | Default |
|-----------|------|----------|---------|
| `input` | string | Yes | - |
| `output` | string | Yes | - |
| `bitrate` | string | No | `192k` |

### ffmpeg_video_to_gif

| Parameter | Type | Required | Default |
|-----------|------|----------|---------|
| `input` | string | Yes | - |
| `output` | string | Yes | - |
| `fps` | int | No | 10 |
| `width` | int | No | - |
| `start_time` | float | No | - |
| `duration` | float | No | - |

### ffmpeg_combine_audio_and_video

| Parameter | Type | Required |
|-----------|------|----------|
| `video_input` | string | Yes |
| `audio_input` | string | Yes |
| `output` | string | Yes |

### ffmpeg_overlay_image_on_video

| Parameter | Type | Required | Default |
|-----------|------|----------|---------|
| `video_input` | string | Yes | - |
| `image_input` | string | Yes | - |
| `output` | string | Yes | - |
| `x` | int | No | 0 |
| `y` | int | No | 0 |
| `scale` | float | No | - |

### ffmpeg_concatenate_media_files

| Parameter | Type | Required |
|-----------|------|----------|
| `inputs` | array | Yes |
| `output` | string | Yes |

### ffmpeg_adjust_volume

| Parameter | Type | Required |
|-----------|------|----------|
| `input` | string | Yes |
| `output` | string | Yes |
| `volume` | string | Yes |

Volume formats: `"0.5"`, `"2.0"`, `"-3dB"`, `"+6dB"`

### ffmpeg_layer_audio_files

| Parameter | Type | Required |
|-----------|------|----------|
| `inputs` | array | Yes |
| `output` | string | Yes |

Each input: `{"path": "...", "offset_seconds": 0, "volume": 1.0}`

## Cloud Storage Support

All tools support GCS URIs:

```json
{
  "input": "gs://bucket/input.wav",
  "output": "gs://bucket/output.mp3"
}
```

## Supported Formats

**Audio:** WAV, MP3, OGG, FLAC, AAC

**Video:** MP4, WebM, MKV, AVI, MOV

**Image:** PNG, JPEG, GIF

## Example Output

Converted from WAV to MP3 using `ffmpeg_convert_audio_wav_to_mp3`:

<audio controls src="../test_output/music_test.mp3">
  <a href="../test_output/music_test.mp3">Download converted MP3</a>
</audio>

## License

Apache-2.0
