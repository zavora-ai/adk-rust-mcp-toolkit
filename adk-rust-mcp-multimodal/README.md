# adk-rust-mcp-multimodal

MCP server for multimodal generation. Part of the ADK Rust MCP toolkit.

## Overview

Provider-agnostic multimodal generation server using large language models with native image and audio capabilities.

**Currently implemented:** Google Gemini API

**Planned:** OpenAI GPT-4o, Anthropic Claude

## Features

- **Image Generation** - Generate images from text prompts
- **Text-to-Speech** - Convert text to speech with style control
- **Voice Selection** - Multiple expressive voices
- **Style Control** - Adjust speech tone (cheerful, calm, etc.)

## Installation

```bash
cargo install adk-rust-mcp-multimodal
```

## Configuration

```bash
export PROJECT_ID=your-gcp-project
export LOCATION=us-central1
```

## Usage

```bash
# Stdio transport
adk-rust-mcp-multimodal

# HTTP transport
adk-rust-mcp-multimodal --transport http --port 8080
```

### MCP Client Configuration

**Important:** The `cwd` field is required for file output with relative paths.

```json
{
  "mcpServers": {
    "multimodal": {
      "command": "/path/to/adk-rust-mcp-multimodal",
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

### multimodal_image_generate

| Parameter | Type | Required | Default |
|-----------|------|----------|---------|
| `prompt` | string | Yes | - |
| `model` | string | No | `gemini-2.0-flash-preview-image-generation` |
| `output_file` | string | No | - |

### multimodal_speech_synthesize

| Parameter | Type | Required | Default |
|-----------|------|----------|---------|
| `text` | string | Yes | - |
| `voice` | string | No | `Kore` |
| `style` | string | No | - |
| `output_file` | string | No | - |

### multimodal_list_voices

List available voices.

## Available Voices

Zephyr, Puck, Charon, Kore, Fenrir, Leda, Orus, Aoede

## Available Styles

neutral, cheerful, sad, angry, fearful, surprised, calm

## Resources

- `multimodal://language_codes` - Supported languages
- `multimodal://voices` - Available voices

## Comparison with Other Servers

| Feature | multimodal | image | speech |
|---------|------------|-------|--------|
| Image Gen | Gemini | Imagen (higher quality) | - |
| TTS | Gemini (style control) | - | Cloud TTS (more voices) |
| Best For | Quick prototyping | Production images | Production TTS |

## License

Apache-2.0
