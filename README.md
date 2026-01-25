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

- Rust 2024 edition (1.85+)
- Google Cloud SDK with authenticated credentials
- FFmpeg installed (for `adk-rust-mcp-avtool`)

### Installation

```bash
# Clone the repository
git clone https://github.com/anthropics/adk-rust-mcp
cd adk-rust-mcp

# Build all servers
cargo build --release
```

### Configuration

Create a `.env` file in the workspace root:

```bash
# Provider configuration (Google Cloud)
PROJECT_ID=your-project-id
LOCATION=us-central1

# Optional: Cloud storage for outputs
GCS_BUCKET=your-bucket-name
```

### Running a Server

All servers support three transport modes:

```bash
# Stdio transport (default) - for local subprocess communication
./target/release/adk-rust-mcp-image

# HTTP Streamable transport (recommended for remote/web clients)
./target/release/adk-rust-mcp-image --transport http --port 8080

# SSE transport
./target/release/adk-rust-mcp-image --transport sse --port 8080
```

## Transport Options

| Transport | Use Case | Command |
|-----------|----------|---------|
| **Stdio** | Local subprocess, Claude Desktop, Kiro | `./adk-rust-mcp-image` |
| **HTTP** | Remote clients, web apps, ADK agents | `./adk-rust-mcp-image --transport http --port 8080` |
| **SSE** | Real-time streaming applications | `./adk-rust-mcp-image --transport sse --port 8080` |

## MCP Client Configuration

### Claude Desktop

Add to `~/.config/claude/claude_desktop_config.json`:

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

### Kiro

Add to `.kiro/settings/mcp.json`:

```json
{
  "mcpServers": {
    "adk-image": {
      "command": "/path/to/adk-rust-mcp-image",
      "args": ["--transport", "stdio"],
      "cwd": "/path/to/workspace",
      "env": {
        "PROJECT_ID": "your-project-id",
        "LOCATION": "us-central1",
        "GCS_BUCKET": "your-bucket-name",
        "RUST_LOG": "info"
      },
      "disabled": false,
      "autoApprove": []
    }
  }
}
```

**Important:** The `cwd` (current working directory) field is required for file output operations with relative paths. Without it, the server may run from a read-only directory (like `/` on macOS) and fail to save files.

### Complete Kiro Configuration Example

Here's a complete configuration for all servers:

```json
{
  "mcpServers": {
    "adk-image": {
      "command": "/path/to/target/release/adk-rust-mcp-image",
      "args": ["--transport", "stdio"],
      "cwd": "/path/to/workspace",
      "env": {
        "PROJECT_ID": "your-project-id",
        "LOCATION": "us-central1",
        "GCS_BUCKET": "your-bucket-name",
        "RUST_LOG": "info"
      }
    },
    "adk-video": {
      "command": "/path/to/target/release/adk-rust-mcp-video",
      "args": ["--transport", "stdio"],
      "cwd": "/path/to/workspace",
      "env": {
        "PROJECT_ID": "your-project-id",
        "LOCATION": "us-central1",
        "GCS_BUCKET": "your-bucket-name",
        "RUST_LOG": "info"
      }
    },
    "adk-music": {
      "command": "/path/to/target/release/adk-rust-mcp-music",
      "args": ["--transport", "stdio"],
      "cwd": "/path/to/workspace",
      "env": {
        "PROJECT_ID": "your-project-id",
        "LOCATION": "us-central1",
        "GCS_BUCKET": "your-bucket-name",
        "RUST_LOG": "info"
      }
    },
    "adk-speech": {
      "command": "/path/to/target/release/adk-rust-mcp-speech",
      "args": ["--transport", "stdio"],
      "cwd": "/path/to/workspace",
      "env": {
        "PROJECT_ID": "your-project-id",
        "LOCATION": "us-central1",
        "RUST_LOG": "info"
      }
    },
    "adk-multimodal": {
      "command": "/path/to/target/release/adk-rust-mcp-multimodal",
      "args": ["--transport", "stdio"],
      "cwd": "/path/to/workspace",
      "env": {
        "PROJECT_ID": "your-project-id",
        "LOCATION": "us-central1",
        "RUST_LOG": "info"
      }
    },
    "adk-avtool": {
      "command": "/path/to/target/release/adk-rust-mcp-avtool",
      "args": ["--transport", "stdio"],
      "cwd": "/path/to/workspace",
      "env": {
        "PROJECT_ID": "your-project-id",
        "LOCATION": "us-central1",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### ADK-Rust Agents (HTTP Transport)

For ADK-Rust agents, use HTTP transport for better reliability:

```rust
use adk_tool::McpHttpClientBuilder;
use std::time::Duration;

// Start server: ./adk-rust-mcp-image --transport http --port 8080

let toolset = McpHttpClientBuilder::new("http://localhost:8080/mcp")
    .timeout(Duration::from_secs(60))
    .connect()
    .await?;
```

See the [examples](./examples/README.md) directory for complete ADK agent examples.

## Examples

The `examples/` directory contains ADK agent examples that demonstrate using MCP servers:

- **image-agent** - Image generation agent
- **video-agent** - Video generation agent  
- **music-agent** - Music composition agent
- **speech-agent** - Text-to-speech agent
- **media-pipeline** - Multi-tool orchestration
- **creative-studio** - Full creative suite

```bash
# Start the MCP server
./target/release/adk-rust-mcp-image --transport http --port 8080

# Run the agent (in another terminal)
cd examples/image-agent
cargo run
```

## Documentation

- [Full Documentation](./docs/README.md)
- [API Reference](./docs/api/README.md)
- [Configuration Guide](./docs/configuration.md)
- [Development Guide](./docs/development.md)
- [Examples](./examples/README.md)

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p adk-rust-mcp-image

# Run integration tests (requires GCP credentials)
cargo test --test integration_test

# Skip integration tests
SKIP_INTEGRATION_TESTS=1 cargo test
```

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

Copyright 2025 Anthropic

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
