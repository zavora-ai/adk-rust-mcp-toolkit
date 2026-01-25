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

### Required Google Cloud APIs

Enable the following APIs for your project:

```bash
# Vertex AI (for image, video, music generation)
gcloud services enable aiplatform.googleapis.com --project=YOUR_PROJECT_ID

# Cloud Text-to-Speech (for speech server)
gcloud services enable texttospeech.googleapis.com --project=YOUR_PROJECT_ID

# Cloud Storage (if using GCS output)
gcloud services enable storage.googleapis.com --project=YOUR_PROJECT_ID
```

### Quota Project Configuration

When using Application Default Credentials (ADC), some Google Cloud APIs require a quota project to be set. If you encounter errors like:

```
PERMISSION_DENIED: Your application is authenticating by using local Application Default Credentials. 
The texttospeech.googleapis.com API requires a quota project, which is not set by default.
```

Set the quota project:

```bash
gcloud auth application-default set-quota-project YOUR_PROJECT_ID
```

Or the servers will automatically include the `x-goog-user-project` header using the `PROJECT_ID` environment variable.

### Verify Setup

```bash
# Check project
gcloud config get-value project

# Test authentication
gcloud auth application-default print-access-token

# Verify APIs are enabled
gcloud services list --enabled --project=YOUR_PROJECT_ID | grep -E "(aiplatform|texttospeech|storage)"
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


## Troubleshooting

### Common Issues

#### "Read-only file system" Error (macOS)

**Symptom:** When saving files with relative paths, you get:
```
Read-only file system (os error 30)
```

**Cause:** The MCP server is running without a working directory set, defaulting to `/` which is read-only on macOS with System Integrity Protection (SIP) enabled.

**Solution:** Add the `cwd` field to your MCP configuration:

```json
{
  "mcpServers": {
    "adk-image": {
      "command": "/path/to/adk-rust-mcp-image",
      "cwd": "/path/to/your/workspace",
      "env": { ... }
    }
  }
}
```

Alternatively, use absolute paths for file output.

#### "PERMISSION_DENIED: quota project not set" Error

**Symptom:** API calls fail with:
```
PERMISSION_DENIED: Your application is authenticating by using local Application Default Credentials. 
The texttospeech.googleapis.com API requires a quota project, which is not set by default.
```

**Cause:** Some Google Cloud APIs (like Cloud TTS) require a quota project to be explicitly set when using ADC.

**Solution:** Set the quota project:
```bash
gcloud auth application-default set-quota-project YOUR_PROJECT_ID
```

Or ensure `PROJECT_ID` is set in your environment - the servers include the `x-goog-user-project` header automatically.

#### "SERVICE_DISABLED" Error

**Symptom:** API calls fail with:
```
"reason": "SERVICE_DISABLED",
"metadata": {
  "service": "texttospeech.googleapis.com"
}
```

**Cause:** The required Google Cloud API is not enabled for your project.

**Solution:** Enable the required API:
```bash
# For speech server
gcloud services enable texttospeech.googleapis.com --project=YOUR_PROJECT_ID

# For image/video/music servers (Vertex AI)
gcloud services enable aiplatform.googleapis.com --project=YOUR_PROJECT_ID
```

#### MCP Server Connection Errors

**Symptom:** Kiro shows "Error connecting to MCP server" with schema validation errors like:
```
invalid_value: expected "object"
```

**Cause:** Tool schemas with no parameters must still include `"type": "object"`.

**Solution:** This is a bug in the server implementation. Ensure tools with no parameters return a schema like:
```json
{
  "type": "object"
}
```

#### Server Not Picking Up Code Changes

**Symptom:** After rebuilding the server, changes don't take effect.

**Cause:** The MCP client (Kiro/Claude Desktop) caches the server process.

**Solution:** 
1. Rebuild the server: `cargo build --release`
2. Restart the MCP server from the client:
   - In Kiro: Use the MCP Server view to disconnect and reconnect
   - Or toggle `"disabled": true` then `"disabled": false` in the config

### Debugging Tips

1. **Enable debug logging:**
   ```json
   "env": {
     "RUST_LOG": "debug"
   }
   ```

2. **Check MCP logs:** In Kiro, open the MCP Logs view to see server output.

3. **Test server directly:**
   ```bash
   # Test that the server starts
   echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}' | \
     PROJECT_ID=your-project ./target/release/adk-rust-mcp-image
   ```

4. **Verify authentication:**
   ```bash
   gcloud auth application-default print-access-token
   ```
