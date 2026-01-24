# ADK MCP Toolkit Examples

This directory contains examples demonstrating how to use ADK-Rust agents with the MCP toolkit servers. Each example showcases natural language interaction with generative media capabilities.

All examples use **HTTP Streamable transport** for MCP communication, which is the recommended approach for ADK agents.

## Prerequisites

1. **Build the MCP servers:**
   ```bash
   cd ..
   cargo build --release
   ```

2. **Set up environment variables:**
   ```bash
   # Create .env file in this directory
   cp .env.example .env
   
   # Edit with your credentials
   # GOOGLE_API_KEY - For Gemini model (agent LLM)
   # PROJECT_ID - For Vertex AI APIs
   # LOCATION - GCP region (default: us-central1)
   # GCS_BUCKET - Optional, for cloud storage output
   ```

3. **Install ADK-Rust dependencies:**
   The examples use local path dependencies to `adk-rust`. Ensure the `adk-rust` repository is cloned at `../../../adk-rust` relative to the examples directory.

## Running Examples

Each example requires starting the MCP server(s) first, then running the agent.

### 1. Image Agent (`image-agent/`)

An agent that generates and upscales images using natural language.

**Terminal 1 - Start the MCP server:**
```bash
cd ..
./target/release/adk-rust-mcp-image --transport http --port 8080
```

**Terminal 2 - Run the agent:**
```bash
cd image-agent
cargo run --release
```

**Environment variables:**
- `MCP_ENDPOINT` - Override default endpoint (default: `http://localhost:8080/mcp`)

**Try these prompts:**
- "Generate a beautiful sunset over mountains"
- "Create a logo for a tech startup called 'NovaTech'"
- "Make a 16:9 landscape of a futuristic city"
- "Upscale the last image to 4x resolution"

### 2. Video Agent (`video-agent/`)

An agent that creates videos from text descriptions.

**Terminal 1 - Start the MCP server:**
```bash
./target/release/adk-rust-mcp-video --transport http --port 8081
```

**Terminal 2 - Run the agent:**
```bash
cd video-agent
cargo run --release
```

**Environment variables:**
- `VIDEO_MCP_ENDPOINT` - Override default endpoint (default: `http://localhost:8081/mcp`)

**Try these prompts:**
- "Create a video of waves crashing on a beach"
- "Generate a drone shot flying over a forest"
- "Make a video from this image with gentle motion"

### 3. Music Agent (`music-agent/`)

An agent that composes music from descriptions.

**Terminal 1 - Start the MCP server:**
```bash
./target/release/adk-rust-mcp-music --transport http --port 8082
```

**Terminal 2 - Run the agent:**
```bash
cd music-agent
cargo run --release
```

**Environment variables:**
- `MUSIC_MCP_ENDPOINT` - Override default endpoint (default: `http://localhost:8082/mcp`)

**Try these prompts:**
- "Compose an upbeat jazz piano piece"
- "Create ambient electronic music for meditation"
- "Generate a rock guitar riff"

### 4. Speech Agent (`speech-agent/`)

An agent that converts text to natural speech.

**Terminal 1 - Start the MCP server:**
```bash
./target/release/adk-rust-mcp-speech --transport http --port 8083
```

**Terminal 2 - Run the agent:**
```bash
cd speech-agent
cargo run --release
```

**Environment variables:**
- `SPEECH_MCP_ENDPOINT` - Override default endpoint (default: `http://localhost:8083/mcp`)

**Try these prompts:**
- "Say 'Welcome to our application' in a cheerful voice"
- "Read this paragraph slowly and clearly"
- "List the available voices"

### 5. Media Pipeline (`media-pipeline/`)

An agent that orchestrates multiple media tools for complex workflows.

**Start all MCP servers (in separate terminals):**
```bash
./target/release/adk-rust-mcp-image --transport http --port 8080
./target/release/adk-rust-mcp-video --transport http --port 8081
./target/release/adk-rust-mcp-music --transport http --port 8082
./target/release/adk-rust-mcp-speech --transport http --port 8083
./target/release/adk-rust-mcp-avtool --transport http --port 8084
```

**Run the agent:**
```bash
cd media-pipeline
cargo run --release
```

**Environment variables:**
- `IMAGE_MCP_ENDPOINT`, `VIDEO_MCP_ENDPOINT`, `MUSIC_MCP_ENDPOINT`, `SPEECH_MCP_ENDPOINT`, `AVTOOL_MCP_ENDPOINT`

**Try these prompts:**
- "Create a video with background music"
- "Generate an image, then create a video from it with narration"
- "Make a GIF from a video and add a watermark"

### 6. Creative Studio (`creative-studio/`)

A comprehensive media agent with access to all creative tools, acting as a creative director.

**Start all MCP servers (same as media-pipeline):**
```bash
./target/release/adk-rust-mcp-image --transport http --port 8080
./target/release/adk-rust-mcp-video --transport http --port 8081
./target/release/adk-rust-mcp-music --transport http --port 8082
./target/release/adk-rust-mcp-speech --transport http --port 8083
./target/release/adk-rust-mcp-avtool --transport http --port 8084
```

**Run the agent:**
```bash
cd creative-studio
cargo run --release
```

**Try these prompts:**
- "I need a complete brand package: logo, jingle, and video intro"
- "Create a podcast intro with music and voice"
- "Design a social media video with animated text"

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    ADK Agent (LLM)                       │
│              Natural Language Understanding              │
├─────────────────────────────────────────────────────────┤
│              McpHttpClientBuilder                        │
│           HTTP Streamable Transport                      │
├─────────────────────────────────────────────────────────┤
│                    McpToolset                            │
│              Tool Discovery & Execution                  │
├──────────┬──────────┬──────────┬──────────┬────────────┤
│  Image   │  Video   │  Music   │  Speech  │  AVTool    │
│  :8080   │  :8081   │  :8082   │  :8083   │  :8084     │
└──────────┴──────────┴──────────┴──────────┴────────────┘
```

## Configuration

### Model Selection

By default, examples use Gemini. To use other models:

```rust
// OpenAI
let model = Arc::new(OpenAIModel::new(&api_key, "gpt-4o")?);

// Anthropic
let model = Arc::new(AnthropicModel::new(&api_key, "claude-sonnet-4")?);

// Ollama (local)
let model = Arc::new(OllamaModel::new("http://localhost:11434", "llama3.2")?);
```

### Custom MCP Endpoints

Override default endpoints using environment variables:

```bash
MCP_ENDPOINT=http://remote-server:8080/mcp cargo run
```

Or configure multiple endpoints:

```bash
IMAGE_MCP_ENDPOINT=http://server1:8080/mcp \
VIDEO_MCP_ENDPOINT=http://server2:8081/mcp \
cargo run
```

## Troubleshooting

### Server not found
Ensure you've built the servers:
```bash
cd .. && cargo build --release
```

### Authentication errors
Check your environment variables:
```bash
echo $GOOGLE_API_KEY
echo $PROJECT_ID
```

### MCP connection refused
Ensure the MCP server is running and listening on the correct port:
```bash
# Check if server is running
curl http://localhost:8080/mcp -X POST \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}'
```

### Timeout errors
For long-running operations (video generation), increase the timeout:
```rust
let toolset = McpHttpClientBuilder::new(&endpoint)
    .timeout(Duration::from_secs(300))  // 5 minutes
    .connect()
    .await?;
```

## Learn More

- [ADK-Rust Documentation](https://github.com/anthropics/adk-rust)
- [MCP Specification](https://modelcontextprotocol.io/)
- [ADK Toolkit Documentation](../docs/README.md)
