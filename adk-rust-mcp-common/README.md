# adk-rust-mcp-common

Shared library for ADK Rust MCP media servers. Part of the [ADK Rust MCP toolkit](https://github.com/zavora-ai/adk-rust-mcp-toolkit).

## Overview

Common infrastructure crate providing configuration, authentication, GCS storage, transport abstraction, and model registry used by all MCP media servers in the workspace.

**Currently implemented:** Google Cloud (Vertex AI, Cloud TTS, Gemini API)

**Planned:** AWS Bedrock, Azure OpenAI, Local/self-hosted models

## Features

- **Configuration** — Environment-based config with dual API support (Gemini API key or Vertex AI ADC)
- **Authentication** — Google Cloud ADC, service account, and API key support
- **GCS Client** — Upload, download, and existence checks for Google Cloud Storage
- **Transport** — MCP transport abstraction (stdio, HTTP, SSE)
- **Server Builder** — Simplified MCP server construction with transport selection
- **Model Registry** — Centralized model definitions, aliases, and capability lookups
- **Error Handling** — Unified error types across all servers
- **Tracing** — Structured logging with optional OpenTelemetry support

## Installation

```toml
[dependencies]
adk-rust-mcp-common = "0.3"
```

## Configuration

```bash
# Option 1: Gemini API (recommended for getting started)
export GEMINI_API_KEY=your-api-key

# Option 2: Vertex AI (for production/enterprise)
export PROJECT_ID=your-gcp-project
export LOCATION=us-central1       # optional, default: us-central1
export GCS_BUCKET=your-bucket     # optional, for cloud storage output
export PORT=8080                  # optional, for HTTP/SSE transport
```

## Usage

### Config

```rust
use adk_rust_mcp_common::Config;

let config = Config::from_env()?;
```

### Authentication

```rust
use adk_rust_mcp_common::auth::AuthProvider;

let auth = AuthProvider::new().await?;
let token = auth.get_token(&["https://www.googleapis.com/auth/cloud-platform"]).await?;
```

### GCS Operations

```rust
use adk_rust_mcp_common::gcs::{GcsClient, GcsUri};

let gcs = GcsClient::with_auth(auth);
let uri = GcsUri::parse("gs://my-bucket/path/to/file.png")?;
gcs.upload(&uri, &data, "image/png").await?;
let data = gcs.download(&uri).await?;
```

### Server Builder

```rust
use adk_rust_mcp_common::{McpServerBuilder, TransportArgs};

McpServerBuilder::new(server)
    .with_transport(transport)
    .run()
    .await?;
```

### Transport Options

| Transport | Use Case | Flag |
|-----------|----------|------|
| Stdio | Claude Desktop, Kiro | `--transport stdio` (default) |
| HTTP | Web apps, ADK agents | `--transport http --port 8080` |
| SSE | Real-time streaming | `--transport sse --port 8080` |

### Model Registry

```rust
use adk_rust_mcp_common::models::ModelRegistry;

let model = ModelRegistry::resolve_imagen("imagen-3");
let model = ModelRegistry::resolve_veo("veo-3.1");
```

## Optional Features

```toml
# OpenTelemetry tracing support
adk-rust-mcp-common = { version = "0.3", features = ["otel"] }
```

## License

Apache-2.0
