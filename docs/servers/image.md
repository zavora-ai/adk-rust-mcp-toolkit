# Image Server (adk-rust-mcp-image)

MCP server for image generation using Google Vertex AI Imagen API.

## Features

- Text-to-image generation with Imagen 3.x and 4.x models
- Multiple aspect ratios (1:1, 3:4, 4:3, 9:16, 16:9)
- Batch generation (1-4 images per request)
- Output to base64, local files, or GCS
- Negative prompts for refined control

## Tools

### image_generate

Generate images from text prompts.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | - | Text description of the image to generate |
| `negative_prompt` | string | No | - | What to avoid in the generated image |
| `model` | string | No | `imagen-4` | Model to use (see Models section) |
| `aspect_ratio` | string | No | `1:1` | Image aspect ratio |
| `number_of_images` | integer | No | `1` | Number of images to generate (1-4) |
| `seed` | integer | No | - | Random seed for reproducibility* |
| `output_file` | string | No | - | Local file path to save image |
| `output_uri` | string | No | - | GCS URI to upload image (gs://bucket/path) |

*Note: Seed is not supported when watermark is enabled (default for Imagen 4).

**Example:**

```json
{
  "prompt": "A serene mountain landscape at sunset with vibrant orange and purple colors",
  "negative_prompt": "blurry, low quality",
  "model": "imagen-4",
  "aspect_ratio": "16:9",
  "number_of_images": 2
}
```

**Response:**

Returns base64-encoded image data, local file paths, or GCS URIs depending on output parameters.

## Resources

### image://models

List available image generation models with their capabilities.

```json
[
  {
    "id": "imagen-4.0-generate-preview-06-06",
    "aliases": ["imagen-4", "imagen-4.0", "imagen4"],
    "max_prompt_length": 2000,
    "supported_aspect_ratios": ["1:1", "3:4", "4:3", "9:16", "16:9"],
    "max_images": 4
  },
  {
    "id": "imagen-3.0-generate-002",
    "aliases": ["imagen-3", "imagen-3.0", "imagen3"],
    "max_prompt_length": 480,
    "supported_aspect_ratios": ["1:1", "3:4", "4:3", "9:16", "16:9"],
    "max_images": 4
  }
]
```

### image://providers

List available image generation providers.

```json
[
  {
    "id": "google-imagen",
    "name": "Google Imagen",
    "description": "Google's Vertex AI Imagen API for high-quality image generation",
    "is_default": true
  }
]
```

### image://segmentation_classes

List segmentation classes for image editing (Google provider specific).

## Models

| Model ID | Aliases | Max Prompt | Description |
|----------|---------|------------|-------------|
| `imagen-4.0-generate-preview-06-06` | `imagen-4`, `imagen-4.0` | 2000 chars | Latest Imagen 4 preview |
| `imagen-3.0-generate-002` | `imagen-3`, `imagen-3.0` | 480 chars | Stable Imagen 3 |
| `imagen-3.0-fast-generate-001` | `imagen-3-fast` | 480 chars | Fast Imagen 3 |

## Usage Examples

### Basic Generation

```bash
# Using MCP client
mcp call image_generate '{"prompt": "A cute cat wearing a hat"}'
```

### Save to Local File

```bash
mcp call image_generate '{
  "prompt": "Abstract art with geometric shapes",
  "output_file": "/tmp/art.png"
}'
```

### Upload to GCS

```bash
mcp call image_generate '{
  "prompt": "Product photo of a coffee mug",
  "output_uri": "gs://my-bucket/images/mug.png"
}'
```

### Multiple Images

```bash
mcp call image_generate '{
  "prompt": "Logo design for a tech startup",
  "number_of_images": 4,
  "aspect_ratio": "1:1"
}'
```

## Error Handling

| Error | Description |
|-------|-------------|
| `INVALID_PARAMS` | Invalid parameters (prompt too long, invalid aspect ratio, etc.) |
| `API_ERROR` | Vertex AI API error (quota exceeded, model unavailable, etc.) |
| `AUTH_ERROR` | Authentication failed |
| `GCS_ERROR` | GCS upload/download failed |

## Configuration

Environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PROJECT_ID` | Yes | - | GCP project ID |
| `LOCATION` | No | `us-central1` | GCP region |
| `GCS_BUCKET` | No | - | Default GCS bucket for output |
| `OTEL_ENABLED` | No | `false` | Enable OpenTelemetry tracing (requires `otel` feature) |
| `OTEL_SERVICE_NAME` | No | `adk-rust-mcp-image` | Service name for tracing |

### OpenTelemetry Tracing

When built with the `otel` feature flag, the server supports optional OpenTelemetry tracing for Google Cloud Trace integration:

```bash
# Build with OpenTelemetry support
cargo build --package adk-rust-mcp-image --features otel

# Enable tracing at runtime
OTEL_ENABLED=true PROJECT_ID=my-project ./adk-rust-mcp-image
```
