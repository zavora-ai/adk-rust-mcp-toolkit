# Video Server (adk-rust-mcp-video)

MCP server for video generation using Google Vertex AI Veo API.

## Features

- Text-to-video generation with Veo 2.x and 3.x models
- Image-to-video generation (single image or interpolation between two frames)
- Video extension (continue existing videos)
- Multiple aspect ratios (16:9, 9:16)
- Configurable duration (4, 6, or 8 seconds)
- Audio generation support (Veo 3.x only)
- Output to GCS with optional local download

## Tools

### video_generate

Generate videos from text prompts.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | - | Text description of the video to generate |
| `model` | string | No | `veo-3.0-generate-preview` | Model to use |
| `aspect_ratio` | string | No | `16:9` | Video aspect ratio (16:9, 9:16) |
| `duration_seconds` | integer | No | `8` | Duration in seconds (4, 6, or 8) |
| `output_gcs_uri` | string | Yes | - | GCS URI for output (required by Veo API) |
| `download_local` | boolean | No | `false` | Download video locally after generation |
| `local_path` | string | No | - | Local path if download_local is true |
| `generate_audio` | boolean | No | - | Generate audio (Veo 3.x only) |
| `seed` | integer | No | - | Random seed for reproducibility |

**Example:**

```json
{
  "prompt": "A drone shot flying over a mountain range at sunset",
  "model": "veo-3",
  "aspect_ratio": "16:9",
  "duration_seconds": 8,
  "output_gcs_uri": "gs://my-bucket/videos/mountain.mp4",
  "generate_audio": true
}
```

### video_from_image

Generate videos from an image (image-to-video).

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `image` | string | Yes | - | Source image (base64, local path, or GCS URI) |
| `prompt` | string | Yes | - | Text describing desired motion |
| `last_frame_image` | string | No | - | Last frame for interpolation mode |
| `model` | string | No | `veo-3.0-generate-preview` | Model to use |
| `aspect_ratio` | string | No | `16:9` | Video aspect ratio |
| `duration_seconds` | integer | No | `8` | Duration in seconds |
| `output_gcs_uri` | string | Yes | - | GCS URI for output |
| `download_local` | boolean | No | `false` | Download locally |
| `local_path` | string | No | - | Local download path |
| `seed` | integer | No | - | Random seed |

### video_extend

Extend an existing video with additional frames.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `video_input` | string | Yes | - | GCS URI of video to extend |
| `prompt` | string | Yes | - | Text describing continuation |
| `model` | string | No | `veo-3.0-generate-preview` | Model to use |
| `duration_seconds` | integer | No | `8` | Extension duration |
| `output_gcs_uri` | string | Yes | - | GCS URI for output |
| `download_local` | boolean | No | `false` | Download locally |
| `local_path` | string | No | - | Local download path |
| `seed` | integer | No | - | Random seed |

## Resources

### video://models

List available video generation models.

### video://providers

List available video generation providers.

## Models

| Model ID | Aliases | Audio Support | Description |
|----------|---------|---------------|-------------|
| `veo-3.0-generate-preview` | `veo-3`, `veo-3.0` | Yes | Latest Veo 3 with audio |
| `veo-2.0-generate-001` | `veo-2`, `veo-2.0` | No | Stable Veo 2 |

## Long-Running Operations

Video generation uses Vertex AI's LRO API since generation takes 2-5 minutes. The server automatically polls with exponential backoff:

- Initial delay: 5 seconds
- Maximum delay: 60 seconds
- Backoff multiplier: 1.5x
- Maximum timeout: ~30 minutes

## Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PROJECT_ID` | Yes | - | GCP project ID |
| `LOCATION` | No | `us-central1` | GCP region |
| `GCS_BUCKET` | No | - | Default GCS bucket |

## Error Handling

| Error | Description |
|-------|-------------|
| `INVALID_PARAMS` | Invalid parameters |
| `API_ERROR` | Vertex AI API error |
| `TIMEOUT` | LRO polling exceeded maximum attempts |
| `GCS_ERROR` | GCS upload/download failed |
