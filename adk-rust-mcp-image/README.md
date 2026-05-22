# adk-rust-mcp-image

MCP server for image generation and upscaling. Part of the [ADK Rust MCP toolkit](https://github.com/zavora-ai/adk-rust-mcp-toolkit).

## Overview

Image generation server supporting text-to-image and image upscaling via the Vertex AI Imagen API. Supports multiple output formats including base64, local file, and cloud storage.

**Currently implemented:** Google Vertex AI (Imagen 3, Imagen 4)

## Features

- **Text-to-Image** — Generate images from text prompts with Imagen 3/4
- **Image Upscaling** — Upscale images 2x or 4x with Imagen 4 Upscale
- **Multiple Outputs** — Generate up to 4 images per request
- **Flexible Output** — Return base64, save to local file, or upload to GCS
- **Aspect Ratios** — 1:1, 3:4, 4:3, 9:16, 16:9
- **Model Aliases** — Use friendly names like `imagen-3` or `imagen-3-fast`
- **Dual API** — Works with Gemini API key or Vertex AI ADC

## Installation

```bash
cargo install adk-rust-mcp-image
```

## Configuration

```bash
# Option 1: Gemini API (recommended for getting started)
export GEMINI_API_KEY=your-api-key

# Option 2: Vertex AI (for production/enterprise)
export PROJECT_ID=your-gcp-project
export LOCATION=us-central1
export GCS_BUCKET=your-bucket  # optional, for cloud storage output
```

## Tools

### image_generate

Generate images from text prompts.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | — | Text describing the image |
| `negative_prompt` | string | No | — | What to avoid |
| `model` | string | No | `imagen-3.0-generate-002` | Model ID or alias |
| `aspect_ratio` | string | No | `1:1` | 1:1, 3:4, 4:3, 9:16, 16:9 |
| `number_of_images` | int | No | 1 | Number of images (1-4) |
| `seed` | int | No | — | Random seed for reproducibility |
| `output_file` | string | No | — | Save to local file path |
| `output_uri` | string | No | — | Upload to GCS (e.g., `gs://bucket/path`) |

### image_upscale

Upscale images to higher resolution.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `image` | string | Yes | — | Source image (base64, local path, or GCS URI) |
| `upscale_factor` | string | No | `x2` | `x2` or `x4` |
| `output_file` | string | No | — | Save to local file path |
| `output_uri` | string | No | — | Upload to GCS |

## Usage Examples

### Generate an image

```bash
# Stdio (default) — for Claude Desktop, Kiro
adk-rust-mcp-image

# HTTP — for web apps, ADK agents
adk-rust-mcp-image --transport http --port 8080

# SSE — for streaming applications
adk-rust-mcp-image --transport sse --port 8080
```

### Example prompts

```
prompt: "A cat sitting in the rain, watercolor style"
aspect_ratio: "16:9"
output_file: "cat_rain.png"
```

```
prompt: "Futuristic city skyline at sunset, cyberpunk aesthetic"
negative_prompt: "blurry, low quality"
number_of_images: 4
```

### Upscale an image

```
image: "cat_rain.png"
upscale_factor: "x4"
output_file: "cat_rain_4x.png"
```

## Supported Models

| Model | ID | Notes |
|-------|-----|-------|
| Imagen 3 | `imagen-3.0-generate-002` | Default, high quality |
| Imagen 3 Fast | `imagen-3.0-fast-generate-001` | Faster, lower cost |

## License

Apache-2.0
