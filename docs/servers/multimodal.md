# Multimodal Server (adk-rust-mcp-multimodal)

MCP server for multimodal generation using Google's Gemini API.

## Features

- Image generation using Gemini's image generation capabilities
- Text-to-speech with style/tone control
- Multiple voice options
- Output to base64 or local files

## Tools

### multimodal_image_generate

Generate images from text prompts using Gemini.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | - | Text prompt describing the image |
| `model` | string | No | `gemini-2.0-flash-preview-image-generation` | Model to use |
| `output_file` | string | No | - | Local file path to save image |

**Example:**

```json
{
  "prompt": "A futuristic cityscape at night with neon lights",
  "output_file": "/tmp/cityscape.png"
}
```

### multimodal_speech_synthesize

Convert text to speech with style control.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `text` | string | Yes | - | Text to synthesize |
| `voice` | string | No | `Kore` | Voice name |
| `style` | string | No | - | Speech style/tone |
| `model` | string | No | `gemini-2.5-flash-preview-tts` | Model to use |
| `output_file` | string | No | - | Local file path to save audio |

**Available Voices:**
- Zephyr, Puck, Charon, Kore, Fenrir, Leda, Orus, Aoede

**Available Styles:**
- neutral, cheerful, sad, angry, fearful, surprised, calm

**Example:**

```json
{
  "text": "Welcome to our service!",
  "voice": "Kore",
  "style": "cheerful",
  "output_file": "/tmp/welcome.wav"
}
```

### multimodal_list_voices

List available Gemini TTS voices.

**Parameters:** None

**Response:**

```json
[
  { "name": "Zephyr", "description": "Gemini TTS voice: Zephyr" },
  { "name": "Kore", "description": "Gemini TTS voice: Kore" }
]
```

## Resources

### multimodal://language_codes

List of supported language codes for Gemini TTS.

### multimodal://voices

List of available Gemini TTS voices.

## Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PROJECT_ID` | Yes | - | GCP project ID |
| `LOCATION` | No | `us-central1` | GCP region |

## Usage Examples

### Generate Image

```bash
mcp call multimodal_image_generate '{
  "prompt": "A serene mountain landscape"
}'
```

### Synthesize Speech with Style

```bash
mcp call multimodal_speech_synthesize '{
  "text": "I am so happy to see you!",
  "voice": "Puck",
  "style": "cheerful"
}'
```

## Error Handling

| Error | Description |
|-------|-------------|
| `INVALID_PARAMS` | Invalid parameters (empty prompt/text, invalid voice/style) |
| `API_ERROR` | Gemini API error |
| `AUTH_ERROR` | Authentication failed |
