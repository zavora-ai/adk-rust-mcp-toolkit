# adk-rust-mcp-speech

MCP server for text-to-speech. Part of the ADK Rust MCP toolkit.

## Overview

Provider-agnostic text-to-speech server designed to support multiple AI backends.

**Currently implemented:** Google Cloud TTS (Chirp3-HD)

**Planned:** AWS Polly, Azure TTS, Local models

## Features

- **High-Quality TTS** - Chirp3-HD voices for natural speech
- **Voice Selection** - Multiple voices with different characteristics
- **Speech Control** - Adjust speaking rate and pitch
- **Custom Pronunciations** - IPA and X-SAMPA phonetic support

## Installation

```bash
cargo install adk-rust-mcp-speech
```

## Configuration

```bash
export PROJECT_ID=your-gcp-project
```

Enable the API:

```bash
gcloud services enable texttospeech.googleapis.com --project=your-project
```

## Usage

```bash
# Stdio transport
adk-rust-mcp-speech

# HTTP transport
adk-rust-mcp-speech --transport http --port 8080
```

## Tools

### speech_synthesize

| Parameter | Type | Required | Default |
|-----------|------|----------|---------|
| `text` | string | Yes | - |
| `voice` | string | No | `en-US-Chirp3-HD-Achernar` |
| `speaking_rate` | float | No | 1.0 |
| `pitch` | float | No | 0.0 |
| `output_file` | string | No | - |

### speech_list_voices

List available voices.

## License

Apache-2.0
