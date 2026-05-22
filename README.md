# ADK Rust MCP Toolkit

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-v1.0-green.svg)](https://modelcontextprotocol.io/)

Production-ready Model Context Protocol (MCP) servers for generative media, built in Rust. Generate images, videos, music, and speech through a unified, provider-agnostic interface.

## Features

- **🖼️ Image Generation** — Text-to-image with Imagen 3 and Gemini multimodal models
- **🎬 Video Generation** — Text-to-video, image-to-video, video extension with Veo 3.1
- **🎵 Music Generation** — Full songs with Lyria 3 Pro/Clip, real-time streaming with Lyria RealTime
- **🗣️ Speech Synthesis** — High-quality TTS with Chirp3-HD and Gemini voices (30 voices, 70+ languages)
- **🎛️ Media Processing** — FFmpeg-powered audio/video manipulation
- **🔌 Multiple Transports** — Stdio, HTTP, and SSE for any integration scenario
- **🔑 Dual API Support** — Works with both Vertex AI (ADC) and Gemini API (API key)

## Example Outputs

<table>
<tr>
<td align="center"><strong>Image Generation</strong></td>
<td align="center"><strong>Multimodal</strong></td>
</tr>
<tr>
<td><img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/cat_rain.png" width="300" alt="Generated cat in rain"/></td>
<td><img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/multimodal_test.png" width="300" alt="Multimodal generation"/></td>
</tr>
</table>

> **Prompt:** "A photorealistic cat sitting in the rain on a cobblestone street, cinematic lighting"

### 🎬 Video Generation

<video src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/video_test.mp4" width="640" controls></video>

> **Prompt:** "A drone shot flying over a misty mountain valley at sunrise, cinematic" — *Model: veo-3.1-generate-preview*

### 🎵 Music Generation

<audio controls src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/music_test.mp3">
  <a href="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/music_test.mp3">Download music sample</a>
</audio>

> **Prompt:** "An epic cinematic orchestral piece about a journey home. Starts with a solo piano intro, builds through sweeping strings." — *Model: lyria-3-pro-preview*

### 🗣️ Speech Synthesis

<audio controls src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/speech_test.wav">
  <a href="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/speech_test.wav">Download speech sample</a>
</audio>

> **Prompt:** "Say cheerfully: Have a wonderful day!" — *Voice: Kore, Model: gemini-2.5-flash-preview-tts*

### 🎭 Meme Generation

<img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/meme_example.png" width="400" alt="Generated meme"/>

> **Prompt:** "a cat sitting at a computer looking confused at code" — *Top: "WHEN THE CODE WORKS", Bottom: "BUT YOU DONT KNOW WHY"*

### 🎞️ GIF Generation

<img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/gif_example.gif" width="400" alt="Generated GIF"/>

> **Prompt:** "A cute robot waving hello, colorful cartoon style, looping animation"

### 📱 Short-Form Video

<video src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/short_example.mp4" width="270" controls></video>

> **Prompt:** "A timelapse of a flower blooming in a garden, vibrant colors, close-up macro shot" — *Caption: "Nature is beautiful 🌸"*

### 🎙️ Podcast Generation

<audio controls src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/podcast_example.wav">
  <a href="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/podcast_example.wav">Download podcast sample</a>
</audio>

> **Script:** Host (Kore): "Welcome to AI Weekly!" → Guest (Puck): "Thanks for having me..." — *Background: "soft ambient electronic"*

### 📊 Presentation Generation

<video src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/presentation_example.mp4" width="640" controls></video>

> **Slides:** "ADK Rust MCP Toolkit" → "Dual API Support" → "Seven Servers" — *Voice: Kore, Music: "soft ambient corporate"*

### 📐 Diagram Generation

<img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/diagram_example.svg" width="600" alt="CI/CD flowchart"/>

> **Prompt:** "CI/CD pipeline: developer pushes code, triggers build, runs tests, if tests pass deploy to staging, then deploy to production" — *Format: SVG, Type: flowchart*

### 🎨 Artist Tools

<table>
<tr>
<td align="center"><strong>Original (Oil Painting)</strong></td>
<td align="center"><strong>Edited (Added Balloon)</strong></td>
</tr>
<tr>
<td><img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/artist_example.png" width="300" alt="Oil painting"/></td>
<td><img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/graphics_example.png" width="300" alt="Edited with balloon"/></td>
</tr>
</table>

> **Artist:** "A peaceful countryside village at sunset with rolling hills and a winding river" — *Style: oil_painting*
>
> **Graphics Edit:** "Add a hot air balloon floating in the sky" — *Input: the oil painting above*

### 📚 Education Tools

**Whiteboard** — Step-by-step math solutions and diagrams:

<img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/education/whiteboard_math.png" width="500" alt="Whiteboard math solution"/>

> **Prompt:** "Solve step by step: 2x + 5 = 15" — *Style: whiteboard, show_steps: true*

**Flashcards** — Visual Q&A cards for any topic:

<table>
<tr>
<td><img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/education/card_01_front.png" width="250" alt="Flashcard front"/></td>
<td><img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/education/card_01_back.png" width="250" alt="Flashcard back"/></td>
</tr>
</table>

> **Topic:** "Solar system planets" — *Count: 3, Age group: 7-9*

**Quiz** — Multiple-choice with illustrated questions:

<img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/education/question_01.png" width="400" alt="Quiz question"/>

> **Topic:** "Dinosaurs" — *Questions: 3, Difficulty: easy, Age group: 8-10*

**Story & Explainer** — Narrated illustrated videos:

<table>
<tr>
<td align="center"><strong>Story</strong><br/><video src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/education/story_turtle.mp4" width="300" controls></video></td>
<td align="center"><strong>Explainer</strong><br/><video src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/education/explainer_water_cycle.mp4" width="300" controls></video></td>
</tr>
</table>

> **Story:** "A brave little turtle who learns to swim" — *Style: watercolor, Voice: Aoede, Moral: "Practice makes perfect"*
>
> **Explainer:** "How does the water cycle work?" — *Style: diagram, Pace: slow, Age group: 8-10*

## Servers

| Server | Description | Tools |
|--------|-------------|-------|
| [`adk-rust-mcp-image`](adk-rust-mcp-image/) | Image generation & upscaling | `image_generate`, `image_upscale` |
| [`adk-rust-mcp-video`](adk-rust-mcp-video/) | Video generation | `video_generate`, `video_from_image`, `video_extend` |
| [`adk-rust-mcp-music`](adk-rust-mcp-music/) | Music generation & real-time streaming | `music_generate`, `music_realtime_start`, `music_realtime_steer`, `music_realtime_stop` |
| [`adk-rust-mcp-speech`](adk-rust-mcp-speech/) | Text-to-speech | `speech_synthesize`, `speech_list_voices` |
| [`adk-rust-mcp-multimodal`](adk-rust-mcp-multimodal/) | Gemini multimodal | `multimodal_image_generate`, `multimodal_speech_synthesize`, `multimodal_list_voices` |
| [`adk-rust-mcp-composer`](adk-rust-mcp-composer/) | Composite media | `gif_generate`, `short_generate`, `meme_generate`, `presentation_generate`, `podcast_generate` |
| [`adk-rust-mcp-education`](adk-rust-mcp-education/) | Educational content | `whiteboard_generate`, `flashcard_generate`, `story_generate`, `quiz_generate`, `explainer_generate` |
| [`adk-rust-mcp-diagrams`](adk-rust-mcp-diagrams/) | Structured diagrams | `diagram_generate`, `diagram_from_code`, `diagram_to_code` |
| [`adk-rust-mcp-artist`](adk-rust-mcp-artist/) | Art creation & style | `artist_create`, `artist_style_transfer`, `artist_sketch_to_art`, `artist_variations` |
| [`adk-rust-mcp-graphics`](adk-rust-mcp-graphics/) | Image editing | `graphics_edit`, `graphics_remove_object`, `graphics_replace_background`, `graphics_resize`, `graphics_enhance` |
| [`adk-rust-mcp-avtool`](adk-rust-mcp-avtool/) | FFmpeg processing | `ffmpeg_*` (8 tools) |

## Quick Start

### Prerequisites

- Rust 1.85+ (2024 edition)
- Google Cloud project with Vertex AI enabled, **or** a Gemini API key
- `gcloud` CLI authenticated (for Vertex AI)
- FFmpeg (for avtool only)

### Installation

```bash
# From crates.io
cargo install adk-rust-mcp-image adk-rust-mcp-video adk-rust-mcp-music \
              adk-rust-mcp-speech adk-rust-mcp-multimodal adk-rust-mcp-avtool

# Or build from source
git clone https://github.com/zavora-ai/adk-rust-mcp-toolkit
cd adk-rust-mcp-toolkit
cargo build --release
```

### Configuration

```bash
# Option 1: Gemini API (recommended for getting started)
export GEMINI_API_KEY=your-api-key  # from https://aistudio.google.com/apikey

# Option 2: Vertex AI (for production/enterprise)
export PROJECT_ID=your-gcp-project
export LOCATION=us-central1
export GCS_BUCKET=your-bucket  # Required for video generation on Vertex
```

The servers auto-detect which API to use: if `GEMINI_API_KEY` is set, they use the Gemini API; otherwise they use Vertex AI with Application Default Credentials.

### Run a Server

```bash
# Stdio (default) — for Claude Desktop, Kiro, local tools
adk-rust-mcp-image

# HTTP — for web apps, remote clients, ADK agents
adk-rust-mcp-image --transport http --port 8080

# SSE — for streaming applications
adk-rust-mcp-image --transport sse --port 8080
```

## Integration

### Kiro

Add to `.kiro/settings/mcp.json`:

```json
{
  "mcpServers": {
    "adk-image": {
      "command": "adk-rust-mcp-image",
      "args": ["--transport", "stdio"],
      "cwd": "/path/to/workspace",
      "env": {
        "PROJECT_ID": "your-project",
        "LOCATION": "us-central1"
      }
    }
  }
}
```

> **Note:** The `cwd` field is required for file output with relative paths.

### Claude Desktop

Add to `~/.config/claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "image": {
      "command": "adk-rust-mcp-image",
      "env": { "PROJECT_ID": "your-project" }
    }
  }
}
```

### HTTP Client (Rust)

```rust
use adk_tool::McpHttpClientBuilder;

let toolset = McpHttpClientBuilder::new("http://localhost:8080/mcp")
    .timeout(Duration::from_secs(120))
    .connect()
    .await?;
```

<details>
<summary><strong>Full Multi-Server Configuration</strong></summary>

```json
{
  "mcpServers": {
    "adk-image": {
      "command": "adk-rust-mcp-image",
      "args": ["--transport", "stdio"],
      "cwd": "/workspace",
      "env": { "PROJECT_ID": "my-project", "LOCATION": "us-central1", "GCS_BUCKET": "my-bucket" }
    },
    "adk-video": {
      "command": "adk-rust-mcp-video",
      "args": ["--transport", "stdio"],
      "cwd": "/workspace",
      "env": { "PROJECT_ID": "my-project", "LOCATION": "us-central1", "GCS_BUCKET": "my-bucket" }
    },
    "adk-music": {
      "command": "adk-rust-mcp-music",
      "args": ["--transport", "stdio"],
      "cwd": "/workspace",
      "env": { "PROJECT_ID": "my-project", "LOCATION": "us-central1" }
    },
    "adk-speech": {
      "command": "adk-rust-mcp-speech",
      "args": ["--transport", "stdio"],
      "cwd": "/workspace",
      "env": { "PROJECT_ID": "my-project" }
    },
    "adk-multimodal": {
      "command": "adk-rust-mcp-multimodal",
      "args": ["--transport", "stdio"],
      "cwd": "/workspace",
      "env": { "PROJECT_ID": "my-project", "LOCATION": "us-central1" }
    },
    "adk-avtool": {
      "command": "adk-rust-mcp-avtool",
      "args": ["--transport", "stdio"],
      "cwd": "/workspace"
    }
  }
}
```

</details>

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     MCP Servers                              │
│  image │ video │ music │ speech │ multimodal │ avtool       │
├─────────────────────────────────────────────────────────────┤
│                  adk-rust-mcp-common                         │
│         Config │ Auth │ GCS │ Transport │ Tracing           │
├─────────────────────────────────────────────────────────────┤
│                  Provider Abstraction                        │
├──────────────┬──────────────┬──────────────┬────────────────┤
│   Google     │    AWS       │   Azure      │    Local       │
│  Vertex AI   │  Bedrock     │  OpenAI      │   Ollama       │
│  Cloud TTS   │   Polly      │   TTS        │   Whisper      │
│   Gemini     │   Nova       │   GPT-4o     │                │
└──────────────┴──────────────┴──────────────┴────────────────┘
```

**Currently Implemented:** Google Cloud (Vertex AI, Cloud TTS, Gemini)

**Planned:** AWS Bedrock, Azure OpenAI, local/self-hosted models

## Documentation

| Resource | Description |
|----------|-------------|
| [Configuration Guide](docs/configuration.md) | Environment variables, authentication |
| [API Reference](docs/api/) | Tool parameters and responses |
| [Server Guides](docs/servers/) | Per-server documentation |
| [Development Guide](docs/development.md) | Contributing, testing, architecture |
| [Examples](examples/) | ADK agent integration examples |

## Testing

```bash
# Unit tests
cargo test --workspace

# Integration tests (requires GCP credentials)
cargo test --workspace --test integration_test

# Skip integration tests
SKIP_INTEGRATION_TESTS=1 cargo test --workspace
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## Security

See [SECURITY.md](SECURITY.md) for reporting vulnerabilities.

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Built with ❤️ by [Zavora AI](https://zavora.ai)
