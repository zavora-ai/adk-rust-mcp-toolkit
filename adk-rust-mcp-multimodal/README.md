# adk-rust-mcp-multimodal

MCP server for Gemini multimodal generation. Part of the [ADK Rust MCP toolkit](https://github.com/zavora-ai/adk-rust-mcp-toolkit).

## Overview

Multimodal generation server using Google's Gemini API for image generation and text-to-speech with style control. Uses Gemini's native multimodal capabilities for quick prototyping and creative workflows.

**Currently implemented:** Google Gemini API (gemini-2.5-flash-image, gemini-2.5-flash-preview-tts)

## Features

- **Image Generation** — Text-to-image with Gemini's native image output
- **Text-to-Speech** — 30 expressive voices with style/tone control
- **Style Control** — Cheerful, calm, sad, angry, fearful, surprised tones
- **30 Voices** — Kore, Puck, Zephyr, Charon, Fenrir, Aoede, and 24 more
- **34 Languages** — Auto-detected from input text
- **Flexible Output** — Return base64 or save to local file
- **Dual API** — Works with Gemini API key or Vertex AI ADC

## Installation

```bash
cargo install adk-rust-mcp-multimodal
```

## Configuration

```bash
# Option 1: Gemini API (recommended for getting started)
export GEMINI_API_KEY=your-api-key

# Option 2: Vertex AI (for production/enterprise)
export PROJECT_ID=your-gcp-project
export LOCATION=us-central1
```

## Tools

### multimodal_image_generate

Generate images from text prompts using Gemini.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | — | Text describing the image |
| `model` | string | No | `gemini-2.5-flash-image` | Model ID |
| `output_file` | string | No | — | Save to local file path |

### multimodal_speech_synthesize

Convert text to speech with style control using Gemini TTS.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `text` | string | Yes | — | Text to synthesize |
| `voice` | string | No | `Kore` | Voice name (30 available) |
| `style` | string | No | — | Delivery style/tone |
| `model` | string | No | `gemini-2.5-flash-preview-tts` | Model ID |
| `output_file` | string | No | — | Save to local file path |

### multimodal_list_voices

List all available Gemini TTS voices.

## Usage Examples

```bash
# Stdio (default) — for Claude Desktop, Kiro
adk-rust-mcp-multimodal

# HTTP — for web apps, ADK agents
adk-rust-mcp-multimodal --transport http --port 8080

# SSE — for streaming applications
adk-rust-mcp-multimodal --transport sse --port 8080
```

### Generate an image

```
prompt: "A serene Japanese garden with cherry blossoms and a koi pond"
output_file: "garden.png"
```

### Synthesize speech with style

```
text: "Great news! Your package has been delivered."
voice: "Puck"
style: "cheerful"
output_file: "notification.wav"
```

## Available Voices

Zephyr, Puck, Charon, Kore, Fenrir, Leda, Orus, Aoede, Callirrhoe, Autonoe, Enceladus, Iapetus, Umbriel, Algieba, Despina, Erinome, Algenib, Rasalgethi, Laomedeia, Achernar, Alnilam, Schedar, Gacrux, Pulcherrima, Achird, Zubenelgenubi, Vindemiatrix, Sadachbia, Sadaltager, Sulafat

## Available Styles

`neutral`, `cheerful`, `sad`, `angry`, `fearful`, `surprised`, `calm`

## Comparison with Other Servers

| Feature | multimodal | image | speech |
|---------|------------|-------|--------|
| Image Gen | Gemini (fast, creative) | Imagen (higher fidelity) | — |
| TTS | Gemini (style control) | — | Cloud TTS (more languages) |
| Best For | Quick prototyping, style TTS | Production images | Production TTS |

## Output Specs

| Output | Format | Details |
|--------|--------|---------|
| Images | PNG | Variable resolution |
| Speech | WAV (PCM) | 24kHz mono, 16-bit |

## License

Apache-2.0
