# adk-rust-mcp-common

Shared utilities and infrastructure for ADK Rust MCP Media servers.

## Overview

This crate provides common functionality used across all MCP servers in the ADK Rust MCP toolkit. It's designed with provider abstraction in mind, enabling support for multiple AI backends.

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

**Currently implemented:** Google Cloud (Vertex AI, Cloud TTS, Gemini)

**Planned:** AWS Bedrock, Azure OpenAI, Local/self-hosted models

## Features

- **Authentication** - Google Cloud ADC and service account support (extensible to other providers)
- **Configuration** - Environment-based configuration loading
- **GCS Client** - Google Cloud Storage upload/download operations
- **Error Handling** - Unified error types across servers
- **Transport** - MCP transport abstraction (stdio, HTTP, SSE)
- **Server Builder** - Simplified MCP server construction
- **Model Registry** - Centralized model definitions and aliases

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
adk-rust-mcp-common = "0.1"
```

## Optional Features

- `otel` - Enable OpenTelemetry tracing support

```toml
[dependencies]
adk-rust-mcp-common = { version = "0.1", features = ["otel"] }
```

## Usage

### Configuration

```rust
use adk_rust_mcp_common::Config;

// Load from environment variables
let config = Config::from_env()?;
println!("Project: {}", config.project_id);
println!("Location: {}", config.location);
```

Required environment variables:
- `PROJECT_ID` - Google Cloud project ID

Optional:
- `LOCATION` - GCP region (default: `us-central1`)
- `GCS_BUCKET` - Default GCS bucket for outputs
- `PORT` - HTTP/SSE server port (default: `8080`)

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

// Parse URI
let uri = GcsUri::parse("gs://my-bucket/path/to/file.png")?;

// Upload
gcs.upload(&uri, &data, "image/png").await?;

// Download
let data = gcs.download(&uri).await?;
```

### MCP Server Builder

```rust
use adk_rust_mcp_common::{McpServerBuilder, TransportArgs};

let server = MyServer::new(config);
let transport = args.transport.into_transport();

McpServerBuilder::new(server)
    .with_transport(transport)
    .run()
    .await?;
```

### Transport Options

All servers support three transport modes:

| Transport | Use Case | Flag |
|-----------|----------|------|
| Stdio | Claude Desktop, Kiro | `--transport stdio` (default) |
| HTTP | Web apps, ADK agents | `--transport http --port 8080` |
| SSE | Real-time streaming | `--transport sse --port 8080` |

### Model Registry

```rust
use adk_rust_mcp_common::models::ModelRegistry;

// Resolve model aliases
let model = ModelRegistry::resolve_imagen("imagen-4");
// Returns: Some(ImagenModel { id: "imagen-4.0-generate-preview-06-06", ... })

let model = ModelRegistry::resolve_veo("veo-3");
// Returns: Some(VeoModel { id: "veo-3.0-generate-preview", ... })
```

## Error Handling

```rust
use adk_rust_mcp_common::error::Error;

// Validation error
return Err(Error::validation("Invalid parameter"));

// API error
return Err(Error::api("https://api.example.com", 400, "Bad request"));

// Storage error
return Err(Error::gcs("Upload failed"));
```

## License

Apache-2.0
