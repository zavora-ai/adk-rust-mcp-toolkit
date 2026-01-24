# ADK MCP Toolkit Examples

This directory contains examples demonstrating how to use ADK-Rust agents with the MCP toolkit servers. Each example showcases natural language interaction with generative media capabilities.

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

## Examples

Each example is a standalone crate. Navigate to the example directory and run:

```bash
cd <example-name>
cargo run
```

### 1. Image Agent (`image-agent/`)

An agent that generates and upscales images using natural language.

```bash
cd image-agent
cargo run
```

**Try these prompts:**
- "Generate a beautiful sunset over mountains"
- "Create a logo for a tech startup called 'NovaTech'"
- "Make a 16:9 landscape of a futuristic city"
- "Upscale the last image to 4x resolution"

### 2. Video Agent (`video-agent/`)

An agent that creates videos from text descriptions.

```bash
cd video-agent
cargo run
```

**Try these prompts:**
- "Create a video of waves crashing on a beach"
- "Generate a drone shot flying over a forest"
- "Make a video from this image with gentle motion"

### 3. Music Agent (`music-agent/`)

An agent that composes music from descriptions.

```bash
cd music-agent
cargo run
```

**Try these prompts:**
- "Compose an upbeat jazz piano piece"
- "Create ambient electronic music for meditation"
- "Generate a rock guitar riff"

### 4. Speech Agent (`speech-agent/`)

An agent that converts text to natural speech.

```bash
cd speech-agent
cargo run
```

**Try these prompts:**
- "Say 'Welcome to our application' in a cheerful voice"
- "Read this paragraph slowly and clearly"
- "List the available voices"

### 5. Media Pipeline (`media-pipeline/`)

An agent that orchestrates multiple media tools for complex workflows.

```bash
cd media-pipeline
cargo run
```

**Try these prompts:**
- "Create a video with background music"
- "Generate an image, then create a video from it with narration"
- "Make a GIF from a video and add a watermark"

### 6. Creative Studio (`creative-studio/`)

A comprehensive media agent with access to all creative tools, acting as a creative director that can handle complex media projects.

```bash
cd creative-studio
cargo run
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
│                    McpToolset                            │
│              Tool Discovery & Execution                  │
├──────────┬──────────┬──────────┬──────────┬────────────┤
│  Image   │  Video   │  Music   │  Speech  │  AVTool    │
│  Server  │  Server  │  Server  │  Server  │  Server    │
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

### Server Paths

Update the server paths in examples if your binaries are in a different location:

```rust
const IMAGE_SERVER: &str = "../target/release/adk-rust-mcp-image";
const VIDEO_SERVER: &str = "../target/release/adk-rust-mcp-video";
// etc.
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

### MCP connection issues
The examples use stdio transport. Ensure the server binaries are executable:
```bash
chmod +x ../target/release/adk-rust-mcp-*
```

## Learn More

- [ADK-Rust Documentation](https://github.com/zavora-ai/adk-rust/wiki)
- [MCP Tools Guide](https://github.com/zavora-ai/adk-rust/wiki/MCP-Tools)
- [ADK Toolkit Documentation](../docs/README.md)
