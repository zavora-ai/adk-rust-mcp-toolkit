# adk-rust-mcp-music

MCP server for music generation and real-time streaming. Part of the ADK Rust MCP toolkit.

## Overview

Music generation server supporting both one-shot generation (Lyria 3 Pro/Clip) and interactive real-time streaming (Lyria RealTime) via the Gemini API.

**Currently implemented:** Google Gemini API (Lyria 3 Pro, Lyria 3 Clip, Lyria RealTime)

## Features

- **Full Songs** — Generate complete songs with vocals, verses, choruses (Lyria 3 Pro, ~2 min)
- **Short Clips** — Quick 30-second clips for loops and previews (Lyria 3 Clip)
- **Real-Time Streaming** — Interactive music generation with live steering via WebSocket
- **Live Controls** — Change prompts, BPM, scale, density, brightness mid-stream
- **Flexible Output** — MP3 (Lyria 3) or WAV (RealTime, 48kHz stereo)
- **Dual API** — Works with Gemini API key or Vertex AI ADC

## Installation

```bash
cargo install adk-rust-mcp-music
```

## Configuration

```bash
# Gemini API (recommended)
export GEMINI_API_KEY=your-api-key

# Or Vertex AI
export PROJECT_ID=your-gcp-project
export LOCATION=us-central1
```

## Tools

### music_generate

Generate music from a text prompt using Lyria 3 Pro or Clip.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `prompt` | string | Yes | Text describing the music |
| `negative_prompt` | string | No | What to avoid |
| `sample_count` | int | No | Number of samples (1-4) |
| `seed` | int | No | Random seed |
| `output_file` | string | No | Save to local file |
| `output_gcs_uri` | string | No | Upload to GCS |

### music_realtime_start

Start a real-time music generation session. Returns a session ID for steering.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `prompts` | array | Yes | Weighted prompts `[{text, weight}]` |
| `config` | object | No | Generation config (BPM, scale, density, etc.) |

### music_realtime_steer

Steer an active session — update prompts, config, or control playback.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | string | Yes | Session ID from start |
| `prompts` | array | No | New weighted prompts |
| `config` | object | No | Updated config |
| `action` | string | No | `"pause"`, `"resume"`, or `"reset_context"` |

**Config options:** `bpm` (60-200), `density` (0-1), `brightness` (0-1), `guidance` (0-6), `scale`, `temperature`, `mute_bass`, `mute_drums`, `only_bass_and_drums`, `music_generation_mode`

### music_realtime_stop

Stop a session and save the accumulated audio as a WAV file.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | string | Yes | Session ID from start |
| `output_file` | string | No | Save WAV to path (48kHz stereo) |

## Usage Examples

### Generate a full song

```
prompt: "An epic cinematic orchestral piece about a journey home. 
Starts with a solo piano intro, builds through sweeping strings."
```

### Real-time DJ session

```
1. Start:  prompts=[{text: "minimal techno, deep bass", weight: 1.0}], config={bpm: 128}
2. Steer:  prompts=[{text: "ambient pads, ethereal", weight: 1.5}, {text: "techno", weight: 0.3}]
3. Steer:  config={bpm: 90, density: 0.3, brightness: 0.8}
4. Stop:   output_file="my_session.wav"
```

### Prompt tips

- **Genre:** "acid jazz", "lo-fi hip hop", "80s synthpop", "Berlin techno"
- **Instruments:** "Rhodes piano", "808 drums", "analog synth pads", "acoustic guitar"
- **Mood:** "dreamy", "aggressive", "ethereal", "upbeat", "melancholic"
- **Structure:** Use `[Verse]`, `[Chorus]`, `[Bridge]` tags for Lyria 3 Pro
- **Instrumental:** Add "Instrumental only, no vocals" for background music

## Output Specs

| Model | Format | Sample Rate | Duration |
|-------|--------|-------------|----------|
| Lyria 3 Pro | MP3 | 44.1kHz | ~2 minutes |
| Lyria 3 Clip | MP3 | 44.1kHz | 30 seconds |
| Lyria RealTime | WAV (PCM) | 48kHz stereo | Unlimited (session-based) |

## License

Apache-2.0
