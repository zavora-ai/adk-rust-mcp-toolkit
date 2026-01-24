# AVTool Server (adk-rust-mcp-avtool)

MCP server for audio/video processing using FFmpeg.

## Features

- Media file information extraction
- Audio format conversion (WAV to MP3)
- Video to GIF conversion
- Audio and video combining
- Image overlay on video
- Media file concatenation
- Volume adjustment
- Audio layering/mixing
- Support for local files and GCS URIs

## Prerequisites

- FFmpeg installed and available in PATH
- FFprobe (included with FFmpeg)

## Tools

### ffmpeg_get_media_info

Get information about a media file.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input` | string | Yes | Input file (local path or GCS URI) |

**Response:**

```json
{
  "duration": 120.5,
  "format": "mp4",
  "streams": [
    {
      "index": 0,
      "codec_type": "video",
      "codec_name": "h264",
      "width": 1920,
      "height": 1080
    },
    {
      "index": 1,
      "codec_type": "audio",
      "codec_name": "aac",
      "sample_rate": 48000,
      "channels": 2
    }
  ]
}
```

### ffmpeg_convert_audio_wav_to_mp3

Convert WAV audio to MP3.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `input` | string | Yes | - | Input WAV file |
| `output` | string | Yes | - | Output MP3 file |
| `bitrate` | string | No | `192k` | Audio bitrate |

### ffmpeg_video_to_gif

Convert video to animated GIF.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `input` | string | Yes | - | Input video file |
| `output` | string | Yes | - | Output GIF file |
| `fps` | integer | No | `10` | Frames per second |
| `width` | integer | No | - | Output width (auto height) |
| `start_time` | float | No | - | Start time in seconds |
| `duration` | float | No | - | Duration in seconds |

### ffmpeg_combine_audio_and_video

Combine separate audio and video files.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `video_input` | string | Yes | Input video file |
| `audio_input` | string | Yes | Input audio file |
| `output` | string | Yes | Output file |

### ffmpeg_overlay_image_on_video

Overlay an image on video.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `video_input` | string | Yes | - | Input video file |
| `image_input` | string | Yes | - | Input image file |
| `output` | string | Yes | - | Output file |
| `x` | integer | No | `0` | X position from left |
| `y` | integer | No | `0` | Y position from top |
| `scale` | float | No | - | Image scale factor |
| `start_time` | float | No | - | When overlay appears |
| `duration` | float | No | - | Overlay duration |

### ffmpeg_concatenate_media_files

Concatenate multiple media files.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `inputs` | array | Yes | List of input files |
| `output` | string | Yes | Output file |

### ffmpeg_adjust_volume

Adjust audio volume.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `input` | string | Yes | Input audio file |
| `output` | string | Yes | Output audio file |
| `volume` | string | Yes | Volume: multiplier (e.g., "0.5", "2.0") or dB (e.g., "-3dB", "+6dB") |

### ffmpeg_layer_audio_files

Layer/mix multiple audio files.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `inputs` | array | Yes | List of audio layers |
| `output` | string | Yes | Output file |

**Audio Layer Object:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `path` | string | Yes | - | Input audio file |
| `offset_seconds` | float | No | `0.0` | Offset from start |
| `volume` | float | No | `1.0` | Volume multiplier |

**Example:**

```json
{
  "inputs": [
    { "path": "/tmp/background.wav", "volume": 0.5 },
    { "path": "/tmp/voice.wav", "offset_seconds": 2.0, "volume": 1.0 }
  ],
  "output": "/tmp/mixed.wav"
}
```

## Resources

The AVTool server does not expose any resources.

## Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PROJECT_ID` | Yes | - | GCP project ID (for GCS access) |
| `GCS_BUCKET` | No | - | Default GCS bucket |

## GCS Support

All tools support both local paths and GCS URIs (`gs://bucket/path`). When using GCS:
- Input files are downloaded to a temp directory
- Output files are uploaded after processing
- Temp files are cleaned up automatically

## Usage Examples

### Get Media Info

```bash
mcp call ffmpeg_get_media_info '{"input": "/tmp/video.mp4"}'
```

### Convert to MP3

```bash
mcp call ffmpeg_convert_audio_wav_to_mp3 '{
  "input": "/tmp/audio.wav",
  "output": "/tmp/audio.mp3",
  "bitrate": "320k"
}'
```

### Create GIF from Video

```bash
mcp call ffmpeg_video_to_gif '{
  "input": "/tmp/video.mp4",
  "output": "/tmp/preview.gif",
  "fps": 15,
  "width": 480,
  "start_time": 5.0,
  "duration": 3.0
}'
```

### Mix Audio Tracks

```bash
mcp call ffmpeg_layer_audio_files '{
  "inputs": [
    {"path": "gs://bucket/music.wav", "volume": 0.3},
    {"path": "gs://bucket/narration.wav", "offset_seconds": 1.0}
  ],
  "output": "gs://bucket/final.wav"
}'
```

## Error Handling

| Error | Description |
|-------|-------------|
| `INVALID_PARAMS` | Invalid parameters |
| `FFMPEG_ERROR` | FFmpeg execution failed |
| `GCS_ERROR` | GCS upload/download failed |
| `FILE_NOT_FOUND` | Input file not found |
