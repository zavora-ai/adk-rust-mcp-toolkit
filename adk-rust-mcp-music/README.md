# adk-rust-mcp-music

MCP server for music generation. Part of the ADK Rust MCP toolkit.

## Overview

Provider-agnostic music generation server designed to support multiple AI backends.

```
┌─────────────────────────────────────────────────┐
│              adk-rust-mcp-music                  │
├─────────────────────────────────────────────────┤
│              Provider Abstraction                │
├──────────┬──────────┬──────────┬────────────────┤
│  Lyria   │  Suno    │ Udio     │   Local        │
│ (Google) │          │          │   Models       │
└──────────┴──────────┴──────────┴────────────────┘
```

**Currently implemented:** Google Vertex AI Lyria

**Planned:** Suno, Udio, Local models

## Features

- **Text-to-Music** - Generate instrumental music from text prompts
- **Multiple Samples** - Generate up to 4 variations per request
- **Negative Prompts** - Exclude unwanted elements
- **Flexible Output** - Return base64, save to local file, or upload to cloud storage
- **High Quality** - 48kHz WAV output, 30-second clips

## Installation

```bash
cargo install adk-rust-mcp-music
```

Or build from source:

```bash
cargo build --release --package adk-rust-mcp-music
```

## Configuration

```bash
export PROJECT_ID=your-gcp-project
export LOCATION=us-central1
export GCS_BUCKET=your-bucket  # optional
```

## Usage

### Running the Server

```bash
# Stdio transport (for Claude Desktop, Kiro)
adk-rust-mcp-music

# HTTP transport (for web clients, ADK agents)
adk-rust-mcp-music --transport http --port 8080
```

### MCP Client Configuration

**Kiro** (`.kiro/settings/mcp.json`):

```json
{
  "mcpServers": {
    "music": {
      "command": "/path/to/adk-rust-mcp-music",
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

### music_generate

Generate instrumental music from text prompts.

| Parameter | Type | Required | Default |
|-----------|------|----------|---------|
| `prompt` | string | Yes | - |
| `negative_prompt` | string | No | - |
| `sample_count` | int | No | 1 |
| `seed` | int | No | - |
| `output_file` | string | No | - |
| `output_gcs_uri` | string | No | - |

## Output Format

- **Format:** WAV
- **Sample Rate:** 48kHz
- **Duration:** 30 seconds per clip
- **Channels:** Stereo

## Prompt Tips

**Good prompts:**
- "Upbeat jazz piano with walking bass and brushed drums"
- "Ambient electronic soundscape with soft pads"
- "Epic orchestral trailer music with brass and percussion"

**Using negative prompts:**
- Exclude vocals: `"negative_prompt": "vocals, singing"`
- Exclude instruments: `"negative_prompt": "drums, percussion"`

## Example Output

<audio controls src="../test_output/music_test.mp3">
  <a href="../test_output/music_test.mp3">Download music sample (MP3)</a>
</audio>

[Download WAV](../test_output/music_test.wav) · [Download MP3](../test_output/music_test.mp3)

## License

Apache-2.0
