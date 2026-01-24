# Video Server API

## Tools

### video_generate

Generate videos from text prompts using Vertex AI Veo.

#### Request Schema

```json
{
  "type": "object",
  "required": ["prompt", "output_gcs_uri"],
  "properties": {
    "prompt": {
      "type": "string",
      "description": "Text prompt describing the video to generate"
    },
    "model": {
      "type": "string",
      "description": "Model to use for generation",
      "default": "veo-3.0-generate-preview",
      "enum": [
        "veo-3.0-generate-preview",
        "veo-2.0-generate-001",
        "veo-3",
        "veo-2"
      ]
    },
    "aspect_ratio": {
      "type": "string",
      "description": "Aspect ratio for the generated video",
      "default": "16:9",
      "enum": ["16:9", "9:16"]
    },
    "duration_seconds": {
      "type": "integer",
      "description": "Duration of the video in seconds",
      "default": 8,
      "enum": [4, 6, 8]
    },
    "output_gcs_uri": {
      "type": "string",
      "description": "GCS URI for output (required by Veo API)",
      "pattern": "^gs://[a-z0-9][a-z0-9._-]*/.*$"
    },
    "download_local": {
      "type": "boolean",
      "description": "Whether to download the video locally after generation",
      "default": false
    },
    "local_path": {
      "type": "string",
      "description": "Local path to save the video if download_local is true"
    },
    "generate_audio": {
      "type": "boolean",
      "description": "Whether to generate audio (only supported on Veo 3.x models)"
    },
    "seed": {
      "type": "integer",
      "description": "Random seed for reproducible generation"
    }
  }
}
```

#### Response

**GCS Output** (default):

```json
{
  "content": [
    {
      "type": "text",
      "text": "Video generated: gs://bucket/path/output.mp4"
    }
  ]
}
```

**With Local Download** (when `download_local: true`):

```json
{
  "content": [
    {
      "type": "text",
      "text": "Video generated: gs://bucket/path/output.mp4\nDownloaded to: /local/path/output.mp4"
    }
  ]
}
```

#### Errors

| Code | Message | Description |
|------|---------|-------------|
| -32602 | Invalid params: prompt cannot be empty | Empty prompt provided |
| -32602 | Invalid params: invalid aspect ratio | Unsupported aspect ratio for model |
| -32602 | Invalid params: duration_seconds must be 4, 6, or 8 | Duration not in valid set |
| -32602 | Invalid params: output_gcs_uri must start with gs:// | Invalid GCS URI format |
| -32602 | Invalid params: generate_audio only supported on Veo 3.x | Audio requested on unsupported model |
| -32603 | API error | Vertex AI API failure |
| -32603 | Timeout | LRO polling exceeded maximum attempts |

---

### video_from_image

Generate videos from an image using Vertex AI Veo (image-to-video). Supports both single-image I2V and interpolation mode (first + last frame).

#### Request Schema

```json
{
  "type": "object",
  "required": ["image", "prompt", "output_gcs_uri"],
  "properties": {
    "image": {
      "type": "string",
      "description": "Source image for video generation (first frame for interpolation). Can be base64 data, local file path, or GCS URI"
    },
    "prompt": {
      "type": "string",
      "description": "Text prompt describing the desired video motion"
    },
    "last_frame_image": {
      "type": "string",
      "description": "Last frame image for interpolation mode. If provided, generates a video interpolating between `image` and `last_frame_image`. Can be base64 data, local file path, or GCS URI"
    },
    "model": {
      "type": "string",
      "description": "Model to use for generation",
      "default": "veo-3.0-generate-preview",
      "enum": [
        "veo-3.0-generate-preview",
        "veo-2.0-generate-001",
        "veo-3",
        "veo-2"
      ]
    },
    "aspect_ratio": {
      "type": "string",
      "description": "Aspect ratio for the generated video",
      "default": "16:9",
      "enum": ["16:9", "9:16"]
    },
    "duration_seconds": {
      "type": "integer",
      "description": "Duration of the video in seconds",
      "default": 8,
      "enum": [4, 6, 8]
    },
    "output_gcs_uri": {
      "type": "string",
      "description": "GCS URI for output (required by Veo API)",
      "pattern": "^gs://[a-z0-9][a-z0-9._-]*/.*$"
    },
    "download_local": {
      "type": "boolean",
      "description": "Whether to download the video locally after generation",
      "default": false
    },
    "local_path": {
      "type": "string",
      "description": "Local path to save the video if download_local is true"
    },
    "seed": {
      "type": "integer",
      "description": "Random seed for reproducible generation"
    }
  }
}
```

#### Response

Same as `video_generate`.

#### Errors

Same as `video_generate`, plus:

| Code | Message | Description |
|------|---------|-------------|
| -32602 | Invalid params: image cannot be empty | Empty image provided |
| -32602 | Invalid params: image file not found | Local image file doesn't exist |
| -32602 | Invalid params: last_frame_image file not found | Last frame image file doesn't exist (interpolation mode) |

---

### video_extend

Extend an existing video by generating additional frames using Vertex AI Veo.

#### Request Schema

```json
{
  "type": "object",
  "required": ["video_input", "prompt", "output_gcs_uri"],
  "properties": {
    "video_input": {
      "type": "string",
      "description": "GCS URI of the video to extend",
      "pattern": "^gs://[a-z0-9][a-z0-9._-]*/.*$"
    },
    "prompt": {
      "type": "string",
      "description": "Text prompt describing the desired continuation"
    },
    "model": {
      "type": "string",
      "description": "Model to use for generation",
      "default": "veo-3.0-generate-preview"
    },
    "duration_seconds": {
      "type": "integer",
      "description": "Duration of the extension in seconds",
      "default": 8,
      "enum": [4, 6, 8]
    },
    "output_gcs_uri": {
      "type": "string",
      "description": "GCS URI for output (required by Veo API)",
      "pattern": "^gs://[a-z0-9][a-z0-9._-]*/.*$"
    },
    "download_local": {
      "type": "boolean",
      "description": "Whether to download the video locally after generation",
      "default": false
    },
    "local_path": {
      "type": "string",
      "description": "Local path to save the video if download_local is true"
    },
    "seed": {
      "type": "integer",
      "description": "Random seed for reproducible generation"
    }
  }
}
```

#### Response

Same as `video_generate`.

#### Errors

Same as `video_generate`, plus:

| Code | Message | Description |
|------|---------|-------------|
| -32602 | Invalid params: video_input must start with gs:// | Invalid GCS URI format for input video |

---

## Resources

### video://models

List available video generation models.

#### Response

```json
[
  {
    "id": "veo-3.0-generate-preview",
    "aliases": ["veo-3", "veo-3.0", "veo3"],
    "supported_aspect_ratios": ["16:9", "9:16"],
    "supported_durations": [4, 6, 8],
    "supports_audio": true
  },
  {
    "id": "veo-2.0-generate-001",
    "aliases": ["veo-2", "veo-2.0", "veo2"],
    "supported_aspect_ratios": ["16:9", "9:16"],
    "supported_durations": [4, 6, 8],
    "supports_audio": false
  }
]
```

### video://providers

List available video generation providers.

#### Response

```json
[
  {
    "id": "google-veo",
    "name": "Google Veo",
    "description": "Google's Vertex AI Veo API for high-quality video generation",
    "is_default": true
  }
]
```

---

## Long-Running Operations

Video generation uses Vertex AI's Long-Running Operations (LRO) API. The server automatically polls for completion using exponential backoff:

- Initial delay: 5 seconds
- Maximum delay: 60 seconds
- Backoff multiplier: 1.5x
- Maximum attempts: 120 (~30 minutes timeout)

The operation status is polled until completion or timeout.
