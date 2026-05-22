# adk-rust-mcp-speech

MCP server for text-to-speech with Chirp3-HD. Part of the [ADK Rust MCP toolkit](https://github.com/zavora-ai/adk-rust-mcp-toolkit).

## Overview

High-quality text-to-speech server using Google Cloud TTS Chirp3-HD voices. Supports 30 voices, 70+ languages, custom pronunciations via IPA/X-SAMPA, and fine-grained control over speaking rate and pitch.

**Currently implemented:** Google Cloud TTS (Chirp3-HD)

## Features

- **Chirp3-HD Voices** — 30 high-quality voices with natural prosody
- **70+ Languages** — Automatic language detection with BCP-47 codes
- **Custom Pronunciations** — IPA and X-SAMPA phonetic alphabets
- **Speech Control** — Adjustable speaking rate (0.25-4.0x) and pitch (-20 to +20 semitones)
- **WAV Output** — High-quality uncompressed audio
- **Flexible Output** — Return base64 or save to local file
- **Dual API** — Works with Gemini API key or Vertex AI ADC

## Installation

```bash
cargo install adk-rust-mcp-speech
```

## Configuration

```bash
# Option 1: Gemini API (recommended for getting started)
export GEMINI_API_KEY=your-api-key

# Option 2: Vertex AI (for production/enterprise)
export PROJECT_ID=your-gcp-project
export LOCATION=us-central1
```

For Vertex AI, enable the Cloud TTS API:

```bash
gcloud services enable texttospeech.googleapis.com --project=your-project
```

## Tools

### speech_synthesize

Convert text to speech using Chirp3-HD voices.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `text` | string | Yes | — | Text to synthesize |
| `voice` | string | No | `en-US-Chirp3-HD-Achernar` | Voice name |
| `language_code` | string | No | `en-US` | BCP-47 language code |
| `speaking_rate` | float | No | 1.0 | Speed (0.25-4.0) |
| `pitch` | float | No | 0.0 | Pitch in semitones (-20 to +20) |
| `pronunciations` | array | No | — | Custom pronunciations |
| `output_file` | string | No | — | Save WAV to local path |

**Pronunciation entry:**

| Field | Type | Description |
|-------|------|-------------|
| `word` | string | Word to customize |
| `phonetic` | string | Phonetic representation |
| `alphabet` | string | `"ipa"` or `"x-sampa"` |

### speech_list_voices

List all available Chirp3-HD voices with their supported languages.

## Usage Examples

```bash
# Stdio (default) — for Claude Desktop, Kiro
adk-rust-mcp-speech

# HTTP — for web apps, ADK agents
adk-rust-mcp-speech --transport http --port 8080

# SSE — for streaming applications
adk-rust-mcp-speech --transport sse --port 8080
```

### Basic synthesis

```
text: "Hello, welcome to the ADK Rust MCP toolkit."
voice: "en-US-Chirp3-HD-Achernar"
output_file: "greeting.wav"
```

### Custom pronunciation

```
text: "The GIF format was created by CompuServe."
pronunciations: [{"word": "GIF", "phonetic": "dʒɪf", "alphabet": "ipa"}]
output_file: "pronunciation_demo.wav"
```

### Multilingual

```
text: "Bonjour, comment allez-vous aujourd'hui?"
language_code: "fr-FR"
output_file: "french_greeting.wav"
```

## Output Specs

| Format | Sample Rate | Channels | Encoding |
|--------|-------------|----------|----------|
| WAV | 24kHz | Mono | 16-bit PCM |

## License

Apache-2.0
