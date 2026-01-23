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

All servers support three transport modes:

### Stdio (Default)

```bash
./adk-rust-mcp-image
# or
./adk-rust-mcp-image --transport stdio
```

### HTTP

```bash
./adk-rust-mcp-image --transport http --port 8080
```

### SSE (Server-Sent Events)

```bash
./adk-rust-mcp-image --transport sse --port 8080
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
```
