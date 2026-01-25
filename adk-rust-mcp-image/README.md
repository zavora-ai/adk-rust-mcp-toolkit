# adk-rust-mcp-image

MCP server for image generation and upscaling. Part of the ADK Rust MCP toolkit.

## Overview

Provider-agnostic image generation server designed to support multiple AI backends.

```
┌─────────────────────────────────────────────────┐
│              adk-rust-mcp-image                  │
├─────────────────────────────────────────────────┤
│              Provider Abstraction                │
├──────────┬──────────┬──────────┬────────────────┤
│  Imagen  │  DALL-E  │ Stable   │   Local        │
│ (Google) │ (OpenAI) │Diffusion │   Models       │
└──────────┴──────────┴──────────┴────────────────┘
```

**Currently implemented:** Google Vertex AI Imagen

**Planned:** OpenAI DALL-E, Stability AI, Local models

## Features

- **Text-to-Image** - Generate images from text prompts
- **Image Upscaling** - Upscale images 2x or 4x
- **Multiple Outputs** - Generate up to 4 images per request
- **Flexible Output** - Return base64, save to local file, or upload to cloud storage
- **Model Aliases** - Use friendly names like `imagen-4` or `imagen-3-fast`

## Installation

```bash
cargo install adk-rust-mcp-image
```

Or build from source:

```bash
cargo build --release --package adk-rust-mcp-image
```

## Configuration

```bash
export PROJECT_ID=your-gcp-project
export LOCATION=us-central1  # optional
export GCS_BUCKET=your-bucket  # optional
```

## Usage

### Running the Server

```bash
# Stdio transport (for Claude Desktop, Kiro)
adk-rust-mcp-image

# HTTP transport (for web clients, ADK agents)
adk-rust-mcp-image --transport http --port 8080
```

### MCP Client Configuration

**Kiro** (`.kiro/settings/mcp.json`):

```json
{
  "mcpServers": {
    "image": {
      "command": "/path/to/adk-rust-mcp-image",
      "args": ["--transport", "stdio"],
      "cwd": "/path/to/workspace",
      "env": {
        "PROJECT_ID": "your-project"
      }
    }
  }
}
```

## Tools

### image_generate

Generate images from text prompts.

| Parameter | Type | Required | Default |
|-----------|------|----------|---------|
| `prompt` | string | Yes | - |
| `negative_prompt` | string | No | - |
| `model` | string | No | `imagen-4` |
| `aspect_ratio` | string | No | `1:1` |
| `number_of_images` | int | No | 1 |
| `output_file` | string | No | - |
| `output_uri` | string | No | - |

### image_upscale

Upscale images to higher resolution.

| Parameter | Type | Required | Default |
|-----------|------|----------|---------|
| `image` | string | Yes | - |
| `upscale_factor` | string | No | `x2` |
| `output_file` | string | No | - |

## Resources

- `image://models` - List available models
- `image://providers` - List providers

## License

Apache-2.0
