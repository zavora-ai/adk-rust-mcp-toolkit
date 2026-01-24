# AVTool Server API

## Tools

### ffmpeg_get_media_info

Get information about a media file using ffprobe.

#### Request Schema

```json
{
  "type": "object",
  "required": ["input"],
  "properties": {
    "input": {
      "type": "string",
      "description": "Input file path (local or GCS URI)"
    }
  }
}
```

#### Response

```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"duration\": 120.5, \"format\": \"mp4\", \"streams\": [...]}"
    }
  ]
}
```

---

### ffmpeg_convert_audio_wav_to_mp3

Convert WAV audio to MP3 format.

#### Request Schema

```json
{
  "type": "object",
  "required": ["input", "output"],
  "properties": {
    "input": {
      "type": "string",
      "description": "Input WAV file path"
    },
    "output": {
      "type": "string",
      "description": "Output MP3 file path"
    },
    "bitrate": {
      "type": "string",
      "description": "Audio bitrate",
      "default": "192k",
      "examples": ["128k", "192k", "256k", "320k"]
    }
  }
}
```

---

### ffmpeg_video_to_gif

Convert video to animated GIF.

#### Request Schema

```json
{
  "type": "object",
  "required": ["input", "output"],
  "properties": {
    "input": {
      "type": "string",
      "description": "Input video file path"
    },
    "output": {
      "type": "string",
      "description": "Output GIF file path"
    },
    "fps": {
      "type": "integer",
      "description": "Frames per second",
      "default": 10
    },
    "width": {
      "type": "integer",
      "description": "Output width (height auto-calculated)"
    },
    "start_time": {
      "type": "number",
      "description": "Start time in seconds"
    },
    "duration": {
      "type": "number",
      "description": "Duration in seconds"
    }
  }
}
```

---

### ffmpeg_combine_audio_and_video

Combine separate audio and video files.

#### Request Schema

```json
{
  "type": "object",
  "required": ["video_input", "audio_input", "output"],
  "properties": {
    "video_input": {
      "type": "string",
      "description": "Input video file path"
    },
    "audio_input": {
      "type": "string",
      "description": "Input audio file path"
    },
    "output": {
      "type": "string",
      "description": "Output file path"
    }
  }
}
```

---

### ffmpeg_overlay_image_on_video

Overlay an image on video.

#### Request Schema

```json
{
  "type": "object",
  "required": ["video_input", "image_input", "output"],
  "properties": {
    "video_input": {
      "type": "string",
      "description": "Input video file path"
    },
    "image_input": {
      "type": "string",
      "description": "Input image file path"
    },
    "output": {
      "type": "string",
      "description": "Output file path"
    },
    "x": {
      "type": "integer",
      "description": "X position from left",
      "default": 0
    },
    "y": {
      "type": "integer",
      "description": "Y position from top",
      "default": 0
    },
    "scale": {
      "type": "number",
      "description": "Image scale factor (e.g., 0.5 for half size)"
    },
    "start_time": {
      "type": "number",
      "description": "When overlay appears (seconds)"
    },
    "duration": {
      "type": "number",
      "description": "Overlay duration (seconds)"
    }
  }
}
```

---

### ffmpeg_concatenate_media_files

Concatenate multiple media files.

#### Request Schema

```json
{
  "type": "object",
  "required": ["inputs", "output"],
  "properties": {
    "inputs": {
      "type": "array",
      "description": "List of input file paths",
      "items": {
        "type": "string"
      },
      "minItems": 1
    },
    "output": {
      "type": "string",
      "description": "Output file path"
    }
  }
}
```

---

### ffmpeg_adjust_volume

Adjust audio volume.

#### Request Schema

```json
{
  "type": "object",
  "required": ["input", "output", "volume"],
  "properties": {
    "input": {
      "type": "string",
      "description": "Input audio file path"
    },
    "output": {
      "type": "string",
      "description": "Output audio file path"
    },
    "volume": {
      "type": "string",
      "description": "Volume adjustment: multiplier (e.g., '0.5', '2.0') or dB (e.g., '-3dB', '+6dB')"
    }
  }
}
```

#### Volume Formats

| Format | Example | Description |
|--------|---------|-------------|
| Multiplier | `"0.5"` | Half volume |
| Multiplier | `"2.0"` | Double volume |
| Decibels | `"-3dB"` | Reduce by 3dB |
| Decibels | `"+6dB"` | Increase by 6dB |

---

### ffmpeg_layer_audio_files

Layer/mix multiple audio files.

#### Request Schema

```json
{
  "type": "object",
  "required": ["inputs", "output"],
  "properties": {
    "inputs": {
      "type": "array",
      "description": "List of audio layers",
      "items": {
        "type": "object",
        "required": ["path"],
        "properties": {
          "path": {
            "type": "string",
            "description": "Input audio file path"
          },
          "offset_seconds": {
            "type": "number",
            "description": "Offset from start",
            "default": 0.0
          },
          "volume": {
            "type": "number",
            "description": "Volume multiplier",
            "default": 1.0
          }
        }
      }
    },
    "output": {
      "type": "string",
      "description": "Output file path"
    }
  }
}
```

---

## Resources

The AVTool server does not expose any resources.

## Common Errors

| Code | Message | Description |
|------|---------|-------------|
| -32602 | Invalid params | Missing or invalid parameters |
| -32603 | ffmpeg failed | FFmpeg execution error |
| -32603 | ffprobe failed | FFprobe execution error |
| -32603 | GCS error | GCS upload/download failed |

## GCS Support

All tools support GCS URIs (`gs://bucket/path`) for both input and output:

- **Input**: Files are downloaded to a temp directory before processing
- **Output**: Files are uploaded to GCS after processing
- **Cleanup**: Temp files are automatically removed

## Supported Formats

### Audio
- WAV, MP3, OGG, FLAC, AAC

### Video
- MP4, WebM, MKV, AVI, MOV

### Image (for overlay)
- PNG, JPEG, GIF
