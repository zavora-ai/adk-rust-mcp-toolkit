# ADK Rust MCP Toolkit

A collection of Model Context Protocol (MCP) servers for generative media, built in Rust. Designed to be provider-agnostic with support for multiple AI backends.

## Overview

This workspace provides MCP servers for various generative AI capabilities:

| Server | Description | Capabilities |
|--------|-------------|--------------|
| `adk-rust-mcp-image` | Image generation & upscaling | Text-to-image, upscaling |
| `adk-rust-mcp-video` | Video generation | Text-to-video, image-to-video |
| `adk-rust-mcp-music` | Music generation | Text-to-music |
| `adk-rust-mcp-speech` | Text-to-speech | Speech synthesis, voice listing |
| `adk-rust-mcp-multimodal` | Multimodal generation | Image gen, TTS with style control |
| `adk-rust-mcp-avtool` | Audio/video processing | Format conversion, mixing, effects |

## Architecture

The toolkit is designed with provider abstraction in mind:

```
┌─────────────────────────────────────────────────┐
│                  MCP Servers                     │
├─────────────────────────────────────────────────┤
│              Provider Abstraction                │
├──────────┬──────────┬──────────┬────────────────┤
│  Google  │   AWS    │  Azure   │   Local/OSS    │
│ Vertex AI│ Bedrock  │ OpenAI   │   Ollama etc   │
└──────────┴──────────┴──────────┴────────────────┘
```

Currently implemented providers:
- **Google Cloud** - Vertex AI (Imagen, Veo, Lyria), Cloud TTS, Gemini

Planned providers:
- AWS Bedrock
- Azure OpenAI
- Local/self-hosted models

## Quick Start

### Prerequisites

- Rust 2024 edition
- Provider credentials (see [Configuration](./docs/configuration.md))
- FFmpeg installed (for `adk-rust-mcp-avtool`)

### Installation

```bash
# Clone the repository
git clone https://github.com/zavora-ai/adk-rust-mcp-toolkit
cd adk-rust-mcp-toolkit

# Build all servers
cargo build --release
```

### Configuration

Create a `.env` file in the workspace root:

```bash
# Provider configuration (Google Cloud example)
PROJECT_ID=your-project-id
LOCATION=us-central1

# Optional: Cloud storage for outputs
GCS_BUCKET=your-bucket-name
```

### Running a Server

```bash
# Stdio transport (default)
./target/release/adk-rust-mcp-image

# HTTP transport
./target/release/adk-rust-mcp-image --transport http --port 8080

# SSE transport
./target/release/adk-rust-mcp-image --transport sse --port 8080
```

## MCP Client Configuration

### Claude Desktop

Add to `~/.config/claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "image-gen": {
      "command": "/path/to/adk-rust-mcp-image"
    }
  }
}
```

### Kiro

Add to `.kiro/settings/mcp.json`:

```json
{
  "mcpServers": {
    "image-gen": {
      "command": "/path/to/adk-rust-mcp-image",
      "env": {
        "PROJECT_ID": "your-project-id"
      }
    }
  }
}
```

## Documentation

- [Full Documentation](./docs/README.md)
- [API Reference](./docs/api/README.md)
- [Configuration Guide](./docs/configuration.md)
- [Development Guide](./docs/development.md)
- [Examples](./examples/README.md) - ADK agent examples using MCP servers

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p adk-rust-mcp-image

# Run with more proptest iterations
PROPTEST_CASES=1000 cargo test --workspace
```

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

Copyright 2025 Zavora Technologies Ltd

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
