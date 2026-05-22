# adk-rust-mcp-graphics

MCP server for AI-powered image editing. Part of the ADK Rust MCP toolkit.

## Overview

Edit images with natural language — remove objects, swap backgrounds, resize with outpainting, enhance quality, and make free-form edits. Powered by Gemini Nano Banana 2.

## Example Output

<table>
<tr>
<td align="center"><strong>Original</strong></td>
<td align="center"><strong>After: "Add a hot air balloon"</strong></td>
</tr>
<tr>
<td><img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/artist_example.png" width="300" alt="Original"/></td>
<td><img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/graphics_example.png" width="300" alt="Edited"/></td>
</tr>
</table>

## Features

- **Natural Language Editing** — Describe changes in plain English
- **Object Removal** — Remove unwanted elements with natural infill
- **Background Replacement** — Keep subject, swap the background
- **Outpainting** — Extend images to new aspect ratios
- **Enhancement** — Sharpen, denoise, upscale, color correct, HDR

## Installation

```bash
cargo install adk-rust-mcp-graphics
```

## Configuration

```bash
export GEMINI_API_KEY=your-api-key  # from https://aistudio.google.com/apikey
```

## Tools

### graphics_edit

Edit an image with natural language instructions.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `image` | string | Yes | — | Source image (file path or base64) |
| `instruction` | string | Yes | — | What to change |
| `output_file` | string | No | — | Save path |

### graphics_remove_object

Remove an object from an image.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `image` | string | Yes | — | Source image |
| `object_to_remove` | string | Yes | — | What to remove |
| `output_file` | string | No | — | Save path |

### graphics_replace_background

Replace the background of an image.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `image` | string | Yes | — | Source image |
| `new_background` | string | Yes | — | Description of new background |
| `output_file` | string | No | — | Save path |

### graphics_resize

Resize/reframe an image to a new aspect ratio (outpainting).

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `image` | string | Yes | — | Source image |
| `aspect_ratio` | string | Yes | — | Target ratio (e.g. "16:9", "9:16") |
| `output_file` | string | No | — | Save path |

### graphics_enhance

Enhance image quality/details.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `image` | string | Yes | — | Source image |
| `enhancement` | string | Yes | — | sharpen, denoise, upscale, color_correct, hdr |
| `output_file` | string | No | — | Save path |

## License

Apache-2.0
