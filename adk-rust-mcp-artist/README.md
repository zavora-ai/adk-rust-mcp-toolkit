# adk-rust-mcp-artist

MCP server for AI art creation and style workflows. Part of the ADK Rust MCP toolkit.

## Overview

Create art in specific styles, transfer styles between images, turn sketches into finished artwork, and generate variations — all powered by Gemini Nano Banana 2.

## Example Output

<table>
<tr>
<td align="center"><strong>Oil Painting</strong></td>
<td align="center"><strong>Style Transfer</strong></td>
</tr>
<tr>
<td><img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/artist_example.png" width="300" alt="Oil painting"/></td>
<td><img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/graphics_example.png" width="300" alt="Edited"/></td>
</tr>
</table>

## Features

- **8 Art Styles** — Oil painting, watercolor, impressionist, abstract, pop art, pencil sketch, digital art, pixel art
- **Style Transfer** — Apply any artistic style to existing images
- **Sketch to Art** — Turn rough sketches into polished artwork
- **Variations** — Generate multiple style interpretations of one image
- **Up to 4K** — Resolution control (1K, 2K, 4K)

## Installation

```bash
cargo install adk-rust-mcp-artist
```

## Configuration

```bash
export GEMINI_API_KEY=your-api-key  # from https://aistudio.google.com/apikey
```

## Tools

### artist_create

Create art from text in a specific style.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | — | Text describing the artwork |
| `style` | string | No | "digital_art" | Art style (see list above) |
| `aspect_ratio` | string | No | "1:1" | e.g. "16:9", "9:16" |
| `resolution` | string | No | "2K" | "1K", "2K", "4K" |
| `output_file` | string | No | — | Save path |

### artist_style_transfer

Apply an art style from one image to another.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `content_image` | string | Yes | — | Source image (file path or base64) |
| `style_description` | string | Yes | — | Description of style to apply |
| `output_file` | string | No | — | Save path |

### artist_sketch_to_art

Turn a rough sketch into finished artwork.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `sketch_image` | string | Yes | — | Sketch image (file path or base64) |
| `description` | string | Yes | — | What the sketch depicts |
| `style` | string | No | "digital_art" | Target art style |
| `output_file` | string | No | — | Save path |

### artist_variations

Generate style variations of an existing image.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `image` | string | Yes | — | Source image (file path or base64) |
| `variations` | int | No | 4 | Number of variations (1-4) |
| `output_dir` | string | No | — | Directory to save variations |

## License

Apache-2.0
