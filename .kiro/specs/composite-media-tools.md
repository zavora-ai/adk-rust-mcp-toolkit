# Composite Media Tools — Implementation Spec

## Overview

Five new composite tools that orchestrate existing MCP servers to produce higher-level media outputs. These tools chain image generation, video generation, TTS, music, and FFmpeg processing into single-call workflows.

All tools live in a new crate: `adk-rust-mcp-composer`

---

## 1. GIF Generator (`gif_generate`)

**Purpose:** Generate animated GIFs from text prompts in a single call.

### Flow
```
Text Prompt → Veo 3.1 Lite (4s video) → FFmpeg video_to_gif → Output GIF
```

### Parameters
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | — | Text describing the animation |
| `duration_seconds` | int | No | 4 | Video duration before conversion (4-8) |
| `fps` | int | No | 12 | GIF frame rate |
| `width` | int | No | 480 | Output width in pixels |
| `loop` | bool | No | true | Whether GIF loops |
| `output_file` | string | No | — | Save path |

### Output
- Animated GIF file or base64-encoded GIF data
- Typical size: 2-8 MB for 4s at 480px

### Implementation Notes
- Use `veo-3.1-lite-generate-preview` for speed (fastest model)
- Download video to temp file, convert with FFmpeg, delete temp
- Apply palette optimization for smaller file size: `palettegen` + `paletteuse` filters

---

## 2. Shorts/Reels Generator (`short_generate`)

**Purpose:** Generate vertical short-form video content with audio, optimized for social media.

### Flow
```
Text Prompt → Veo 3.1 (9:16, 8s, with audio) → Optional: text overlay via FFmpeg → Output MP4
```

### Parameters
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | — | Video content description |
| `caption` | string | No | — | Text overlay (bottom of screen) |
| `caption_style` | string | No | "bold_white" | Caption style: bold_white, outline, shadow |
| `duration_seconds` | int | No | 8 | Duration (4-8) |
| `model` | string | No | "veo-3.1-generate-preview" | Veo model |
| `generate_audio` | bool | No | true | Include AI-generated audio |
| `output_file` | string | No | — | Save path |

### Output
- Vertical MP4 (1080x1920 or 720x1280), 9:16 aspect ratio
- With synchronized audio (Veo 3.1 native audio)
- Optional burned-in caption text

### Implementation Notes
- Always use `aspect_ratio: "9:16"` for vertical
- Caption overlay uses FFmpeg `drawtext` filter with configurable font/position
- Default caption position: bottom 15%, centered, with semi-transparent background

---

## 3. Meme Generator (`meme_generate`)

**Purpose:** Generate meme images with top/bottom text from a prompt or template.

### Flow
```
Option A: Text Prompt → Gemini Image Gen → FFmpeg text overlay → Output PNG
Option B: Template name + top/bottom text → Fetch template → FFmpeg text overlay → Output PNG
```

### Parameters
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes* | — | Image description (*or use `template`) |
| `template` | string | No | — | Meme template name (e.g., "drake", "distracted_boyfriend") |
| `top_text` | string | No | — | Top text |
| `bottom_text` | string | No | — | Bottom text |
| `font_size` | int | No | 48 | Text size |
| `output_file` | string | No | — | Save path |

### Output
- PNG or JPEG image with meme text overlay
- Standard meme format: Impact font, white text with black outline

### Implementation Notes
- If `prompt` provided: generate image with Gemini, then overlay text
- If `template` provided: use a bundled set of popular meme templates (or generate from description)
- Text rendering: FFmpeg `drawtext` with `borderw=3:bordercolor=black:fontcolor=white:fontfile=Impact`
- Auto-wrap long text to fit image width
- Consider: let Gemini generate the meme directly by including text instructions in the prompt (simpler, less control)

---

## 4. Slideshow/Presentation Generator (`presentation_generate`)

**Purpose:** Generate a narrated video presentation from structured content (slides with bullet points).

### Flow
```
Slides Input → For each slide:
  ├── Gemini Image Gen (visual for slide)
  ├── Gemini TTS (narration for slide)
  └── Lyria 3 Clip (background music, once)
→ FFmpeg: combine images + audio + music → Output MP4
```

### Parameters
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `slides` | array | Yes | — | Array of slide objects |
| `slides[].title` | string | Yes | — | Slide title |
| `slides[].content` | string | Yes | — | Narration text / bullet points |
| `slides[].image_prompt` | string | No | — | Custom image prompt (auto-generated if omitted) |
| `slides[].duration` | float | No | auto | Seconds to show slide (auto = TTS duration + 1s) |
| `style` | string | No | "professional" | Visual style: professional, playful, minimal, dark |
| `voice` | string | No | "Kore" | TTS voice name |
| `background_music` | string | No | — | Music prompt (e.g., "soft corporate background") |
| `music_volume` | float | No | 0.15 | Background music volume (0-1) |
| `transition` | string | No | "crossfade" | Transition: crossfade, cut, fade_black |
| `output_file` | string | Yes | — | Output MP4 path |

### Slide Object
```json
{
  "title": "Introduction",
  "content": "Welcome to our quarterly review. Today we'll cover three key areas.",
  "image_prompt": "A modern office with charts on screens, professional, clean"
}
```

### Output
- MP4 video (16:9, 1280x720 or 1920x1080)
- Each slide: generated image + TTS narration
- Optional background music mixed at low volume
- Crossfade transitions between slides

### Implementation Notes
- Generate all images in parallel (batch Gemini calls)
- Generate all TTS in parallel
- Generate background music once (Lyria 3 Clip, 30s, loop if needed)
- Use FFmpeg to:
  1. Create video from each image (static frame for duration)
  2. Concatenate with crossfade transitions
  3. Mix narration audio track
  4. Layer background music at reduced volume
- If `image_prompt` is omitted, auto-generate from: `"{style} illustration for a presentation slide about: {title}. {content}"`
- Total duration = sum of slide durations

---

## 5. Podcast/Audio Story Generator (`podcast_generate`)

**Purpose:** Generate multi-speaker audio content with background music — podcasts, dialogues, audio stories.

### Flow
```
Script Input → Gemini TTS (multi-speaker) → Lyria 3 (background music) → FFmpeg layer → Output
```

### Parameters
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `script` | array | Yes | — | Array of dialogue segments |
| `script[].speaker` | string | Yes | — | Speaker name |
| `script[].text` | string | Yes | — | What they say |
| `script[].voice` | string | No | auto | Voice name (auto-assigned per speaker) |
| `script[].style` | string | No | — | Delivery style (e.g., "excited", "whispered") |
| `title` | string | No | — | Episode title (for metadata) |
| `background_music` | string | No | — | Music prompt |
| `music_volume` | float | No | 0.1 | Background music volume |
| `intro_music` | bool | No | false | Add 3s music intro before dialogue |
| `output_file` | string | Yes | — | Output path (.wav or .mp3) |

### Script Segment
```json
[
  {"speaker": "Host", "text": "Welcome to the show! Today we're talking about AI music.", "style": "cheerful"},
  {"speaker": "Guest", "text": "Thanks for having me. It's an exciting time.", "voice": "Puck"},
  {"speaker": "Host", "text": "Let's dive right in. What got you started?"}
]
```

### Output
- WAV or MP3 audio file
- Multi-speaker dialogue with natural voices
- Optional background music layered underneath

### Implementation Notes
- Auto-assign voices to speakers if not specified (round-robin from: Kore, Puck, Zephyr, Charon, Aoede, Fenrir)
- Use Gemini 3.1 Flash TTS multi-speaker mode for natural conversation flow
- If script is short enough (< 5000 chars), send as single multi-speaker TTS call
- If long, batch into chunks and concatenate
- Generate background music with Lyria 3 Clip (30s), loop to match dialogue length
- Layer with FFmpeg: dialogue at full volume, music at `music_volume`
- Add 0.5s silence between segments for natural pacing
- If `intro_music` is true, prepend 3s of music before dialogue starts

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  adk-rust-mcp-composer                       │
│  gif │ short │ meme │ presentation │ podcast                │
├─────────────────────────────────────────────────────────────┤
│              Orchestration Layer                             │
│         (calls other servers via internal API)              │
├──────────┬──────────┬──────────┬──────────┬─────────────────┤
│  Video   │  Image   │  Music   │   TTS    │    FFmpeg       │
│  (Veo)   │ (Gemini) │ (Lyria)  │ (Gemini) │   (avtool)     │
└──────────┴──────────┴──────────┴──────────┴─────────────────┘
```

### Internal Communication

The composer crate imports the handler libraries directly (no network calls):
```rust
use adk_rust_mcp_video::VideoHandler;
use adk_rust_mcp_image::ImageHandler;  // or multimodal
use adk_rust_mcp_music::MusicHandler;
use adk_rust_mcp_multimodal::MultimodalHandler;
```

This avoids MCP-over-MCP overhead and keeps everything in-process.

### Temp File Management

All intermediate files use `tempfile` crate:
```rust
let temp_dir = tempfile::tempdir()?;
let video_path = temp_dir.path().join("input.mp4");
// ... process ...
// temp_dir auto-deleted on drop
```

---

## Crate Structure

```
adk-rust-mcp-composer/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── server.rs          # MCP ServerHandler with 5 tools
│   ├── gif.rs             # gif_generate implementation
│   ├── short.rs           # short_generate implementation
│   ├── meme.rs            # meme_generate implementation
│   ├── presentation.rs   # presentation_generate implementation
│   └── podcast.rs         # podcast_generate implementation
└── tests/
    └── integration_test.rs
```

### Dependencies
```toml
[dependencies]
adk-rust-mcp-common = { path = "../adk-rust-mcp-common" }
adk-rust-mcp-video = { path = "../adk-rust-mcp-video" }
adk-rust-mcp-multimodal = { path = "../adk-rust-mcp-multimodal" }
adk-rust-mcp-music = { path = "../adk-rust-mcp-music" }
adk-rust-mcp-avtool = { path = "../adk-rust-mcp-avtool" }
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
tempfile = "3"
# ... standard deps
```

---

## Priority Order

1. **GIF** — Simplest, one chain (video → gif), immediate value
2. **Shorts** — Simple, one call + optional overlay
3. **Meme** — Image gen + text overlay, fun and viral
4. **Podcast** — Multi-speaker TTS + music layering
5. **Presentation** — Most complex, parallel generation + assembly

---

## Success Criteria

- Each tool produces output in a single MCP tool call
- No intermediate files left behind (temp cleanup)
- Reasonable timeouts (GIF: 60s, Short: 120s, Meme: 30s, Presentation: 5min, Podcast: 3min)
- Error messages clearly indicate which stage failed
- All tools work with Gemini API (primary) and Vertex AI (fallback)
