# Configuration

All MCP servers in this workspace share common configuration through environment variables.

## Environment Variables

### Required

| Variable | Description |
|----------|-------------|
| `PROJECT_ID` | Google Cloud project ID |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `LOCATION` | `us-central1` | Google Cloud region for Vertex AI |
| `GCS_BUCKET` | - | GCS bucket for media output |
| `PORT` | `8080` | HTTP/SSE server port |
| `RUST_LOG` | `info` | Logging level (trace, debug, info, warn, error) |

### Provider-Specific (Future)

| Variable | Description |
|----------|-------------|
| `OPENAI_API_KEY` | OpenAI API key for DALL-E provider |
| `REPLICATE_API_TOKEN` | Replicate API token |
| `S3_BUCKET` | AWS S3 bucket for storage |

## .env File

Create a `.env` file in the workspace root:

```bash
# Google Cloud Configuration
PROJECT_ID=your-project-id
LOCATION=us-central1

# Storage
GCS_BUCKET=your-media-bucket

# Server
PORT=8080

# Logging
RUST_LOG=info
```

The servers automatically load this file using `dotenvy`.

## Authentication

### Google Cloud

The servers use Application Default Credentials (ADC). Set up authentication:

```bash
# Option 1: User credentials (development)
gcloud auth application-default login

# Option 2: Service account (production)
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
```

Required IAM roles:
- `roles/aiplatform.user` - For Vertex AI API access
- `roles/storage.objectAdmin` - For GCS read/write (if using GCS output)

### Verify Setup

```bash
# Check project
gcloud config get-value project

# Test authentication
gcloud auth application-default print-access-token
```

## Transport Options

All servers support three transport modes via command-line arguments:

### Stdio (Default)

Best for local subprocess communication (Claude Desktop, Kiro):

```bash
./adk-rust-mcp-image
# or explicitly
./adk-rust-mcp-image --transport stdio
```

### HTTP Streamable

Best for remote clients, web applications, and ADK agents:

```bash
./adk-rust-mcp-image --transport http --port 8080
```

The MCP endpoint is available at `/mcp` (e.g., `http://localhost:8080/mcp`).

### SSE (Server-Sent Events)

For real-time streaming applications:

```bash
./adk-rust-mcp-image --transport sse --port 8080
```

## Port Configuration

When running multiple servers, use different ports:

```bash
./adk-rust-mcp-image --transport http --port 8080
./adk-rust-mcp-video --transport http --port 8081
./adk-rust-mcp-music --transport http --port 8082
./adk-rust-mcp-speech --transport http --port 8083
./adk-rust-mcp-avtool --transport http --port 8084
```

Or use the `PORT` environment variable:

```bash
PORT=9000 ./adk-rust-mcp-image --transport http
```

## Logging

Control logging with `RUST_LOG`:

```bash
# All modules at info level
RUST_LOG=info ./adk-rust-mcp-image

# Debug for specific module
RUST_LOG=adk_rust_mcp_image=debug ./adk-rust-mcp-image

# Multiple levels
RUST_LOG=info,adk_rust_mcp_common=debug ./adk-rust-mcp-image

# Trace level for detailed debugging
RUST_LOG=trace ./adk-rust-mcp-image
```

## OpenTelemetry Tracing (Optional)

When built with the `otel` feature, servers support OpenTelemetry tracing:

```bash
# Build with OpenTelemetry support
cargo build --package adk-rust-mcp-image --features otel

# Enable tracing at runtime
OTEL_ENABLED=true PROJECT_ID=my-project ./adk-rust-mcp-image
```

Environment variables for OpenTelemetry:
- `OTEL_ENABLED` - Enable/disable tracing (default: false)
- `OTEL_SERVICE_NAME` - Service name for traces (default: server name)
