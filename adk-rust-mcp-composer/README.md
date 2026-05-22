# adk-rust-mcp-composer

MCP server for composite media generation. Part of the [ADK Rust MCP toolkit](https://github.com/zavora-ai/adk-rust-mcp-toolkit).

## Overview

Composite media server that orchestrates multiple AI models (Veo, Gemini, Lyria) and FFmpeg to produce rich media outputs — animated GIFs, short-form videos, memes, narrated presentations, and multi-speaker podcasts.

**Currently implemented:** Google Gemini API (Veo 3.1, Gemini 2.5 Flash, Lyria 3 Clip)

## Features

- **GIF Generation** — Text-to-GIF via Veo video + FFmpeg palette optimization
- **Short-Form Video** — Vertical 9:16 videos with optional caption overlay
- **Meme Generation** — AI image with baked-in meme text (Impact style)
- **Presentations** — Slide-based narrated videos with images, TTS, and background music
- **Podcasts** — Multi-speaker dialogue with auto-assigned voices and background music
- **Dual API** — Works with Gemini API key or Vertex AI ADC

## Prerequisites

FFmpeg must be installed (for GIF conversion, captions, and audio mixing):

```bash
# macOS
brew install ffmpeg

# Ubuntu/Debian
sudo apt install ffmpeg
```

## Installation

```bash
cargo install adk-rust-mcp-composer
```

## Configuration

```bash
# Gemini API (required)
export GEMINI_API_KEY=your-api-key

# Or Vertex AI
export PROJECT_ID=your-gcp-project
export LOCATION=us-central1
```

## Tools

### gif_generate

Generate an animated GIF from a text prompt. Uses Veo for video generation then converts to optimized GIF.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | — | Text describing the animation |
| `duration_seconds` | int | No | 4 | Duration (4-8 seconds) |
| `fps` | int | No | 12 | GIF frame rate |
| `width` | int | No | 480 | Output width in pixels |
| `output_file` | string | No | — | Output file path |

### short_generate

Generate a vertical short-form video (9:16) with audio and optional caption overlay. Optimized for social media.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | — | Video content description |
| `caption` | string | No | — | Text overlay (bottom of screen) |
| `duration_seconds` | int | No | 8 | Duration (4-8 seconds) |
| `generate_audio` | bool | No | true | Generate audio with video |
| `output_file` | string | No | — | Output file path |

### meme_generate

Generate a meme image with top/bottom text overlay from a text prompt.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | — | Image description (what the meme shows) |
| `top_text` | string | No | — | Top text |
| `bottom_text` | string | No | — | Bottom text |
| `font_size` | int | No | 48 | Font size |
| `output_file` | string | No | — | Output file path |

### presentation_generate

Generate a narrated video presentation from slides. Each slide gets an AI image, TTS narration, and optional background music.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `slides` | array | Yes | — | Array of slide objects |
| `style` | string | No | `professional` | Visual style for images |
| `voice` | string | No | `Kore` | TTS voice name |
| `background_music` | string | No | — | Music prompt (Lyria 3 Clip) |
| `music_volume` | float | No | 0.15 | Music volume (0.0-1.0) |
| `output_file` | string | Yes | — | Output MP4 file path |

**Slide format:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | Yes | Slide title |
| `content` | string | Yes | Narration text |
| `image_prompt` | string | No | Custom image prompt (auto-generated if omitted) |

### podcast_generate

Generate a multi-speaker podcast/dialogue with background music. Each speaker gets a unique voice.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `script` | array | Yes | — | Dialogue script segments |
| `background_music` | string | No | — | Music prompt (Lyria 3 Clip) |
| `music_volume` | float | No | 0.1 | Music volume (0.0-1.0) |
| `intro_music` | bool | No | false | Add 3s music intro |
| `output_file` | string | Yes | — | Output file path |

**Script segment format:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `speaker` | string | Yes | Speaker name |
| `text` | string | Yes | What they say |
| `voice` | string | No | Voice name (auto-assigned if omitted) |
| `style` | string | No | Delivery style (cheerful, serious, etc.) |

## Usage Examples

```bash
# Stdio (default) — for Claude Desktop, Kiro
adk-rust-mcp-composer

# HTTP — for web apps, ADK agents
adk-rust-mcp-composer --transport http --port 8080

# SSE — for streaming applications
adk-rust-mcp-composer --transport sse --port 8080
```

### Generate a GIF

```
prompt: "A cat chasing a laser pointer across a living room"
fps: 15
width: 640
output_file: "cat_laser.gif"
```

### Create a short-form video

```
prompt: "Satisfying coffee pour in slow motion, steam rising"
caption: "Monday motivation ☕"
duration_seconds: 6
output_file: "coffee_short.mp4"
```

### Generate a meme

```
prompt: "A developer staring at a screen with code"
top_text: "IT WORKS ON MY MACHINE"
bottom_text: "SHIPS TO PRODUCTION"
output_file: "dev_meme.png"
```

### Create a presentation

```json
{
  "slides": [
    {"title": "Introduction", "content": "Welcome to our quarterly review."},
    {"title": "Results", "content": "Revenue grew 25% year over year."},
    {"title": "Next Steps", "content": "We plan to expand into three new markets."}
  ],
  "voice": "Puck",
  "background_music": "Soft corporate background music, upbeat and professional",
  "output_file": "quarterly_review.mp4"
}
```

### Generate a podcast

```json
{
  "script": [
    {"speaker": "Host", "text": "Welcome to Tech Talk! Today we're discussing AI.", "style": "cheerful"},
    {"speaker": "Guest", "text": "Thanks for having me. AI is transforming everything.", "style": "calm"},
    {"speaker": "Host", "text": "Let's dive into the latest developments."}
  ],
  "background_music": "Lo-fi podcast background music, subtle and warm",
  "intro_music": true,
  "output_file": "tech_talk.wav"
}
```

## Output Specs

| Tool | Format | Details |
|------|--------|---------|
| gif_generate | GIF | Palette-optimized, configurable FPS/width |
| short_generate | MP4 (H.264) | 9:16 vertical, with audio |
| meme_generate | PNG | Square (1:1) with text overlay |
| presentation_generate | MP4 (H.264) | Slide images + narration + music |
| podcast_generate | WAV | 24kHz mono, multi-speaker dialogue |

## License

Apache-2.0
