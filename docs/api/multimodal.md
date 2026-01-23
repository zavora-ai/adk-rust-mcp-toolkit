# Multimodal Server API

The Multimodal server provides image generation and text-to-speech using Google's Gemini API.

## Tools

### multimodal_image_generate

Generate images from a text prompt using Google's Gemini API.

**Parameters:**

| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `prompt` | string | Yes | - | Text prompt describing the image to generate |
| `model` | string | No | `gemini-2.0-flash-preview-image-generation` | Model to use for generation |
| `output_file` | string | No | - | Local file path to save the image |

**Output:**
- If `output_file` is not specified: Returns base64-encoded image data with MIME type
- If `output_file` is specified: Saves image to local path and returns confirmation message

**Example:**
```json
{
  "prompt": "A serene mountain landscape at sunset",
  "output_file": "/tmp/landscape.png"
}
```

---

### multimodal_speech_synthesize

Convert text to speech using Google's Gemini API with style/tone control.

**Parameters:**

| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `text` | string | Yes | - | Text to synthesize into speech |
| `voice` | string | No | `Kore` | Voice name to use |
| `style` | string | No | - | Style/tone for the speech |
| `model` | string | No | `gemini-2.5-flash-preview-tts` | Model to use for TTS |
| `output_file` | string | No | - | Local file path to save the audio |

**Available Voices:**
- Zephyr, Puck, Charon, Kore, Fenrir, Leda, Orus, Aoede

**Available Styles:**
- neutral, cheerful, sad, angry, fearful, surprised, calm

**Output:**
- If `output_file` is not specified: Returns base64-encoded audio data as data URI
- If `output_file` is specified: Saves audio to local path and returns confirmation message

**Example:**
```json
{
  "text": "Hello, welcome to our application!",
  "voice": "Kore",
  "style": "cheerful",
  "output_file": "/tmp/greeting.wav"
}
```

---

### multimodal_list_voices

List available Gemini TTS voices.

**Parameters:** None

**Output:** JSON array of voice objects with `name` and `description` fields.

**Example Response:**
```json
[
  { "name": "Zephyr", "description": "Gemini TTS voice: Zephyr" },
  { "name": "Kore", "description": "Gemini TTS voice: Kore" }
]
```

---

## Resources

### multimodal://language_codes

List of supported language codes for Gemini TTS.

**MIME Type:** `application/json`

**Response:**
```json
[
  { "code": "en-US", "name": "English (US)" },
  { "code": "en-GB", "name": "English (UK)" },
  { "code": "es-ES", "name": "Spanish (Spain)" },
  { "code": "fr-FR", "name": "French (France)" },
  { "code": "de-DE", "name": "German (Germany)" },
  { "code": "ja-JP", "name": "Japanese (Japan)" },
  { "code": "zh-CN", "name": "Chinese (Simplified)" }
]
```

### multimodal://voices

List of available Gemini TTS voices.

**MIME Type:** `application/json`

**Response:**
```json
[
  { "name": "Zephyr", "description": "Gemini TTS voice: Zephyr" },
  { "name": "Puck", "description": "Gemini TTS voice: Puck" },
  { "name": "Charon", "description": "Gemini TTS voice: Charon" },
  { "name": "Kore", "description": "Gemini TTS voice: Kore" },
  { "name": "Fenrir", "description": "Gemini TTS voice: Fenrir" },
  { "name": "Leda", "description": "Gemini TTS voice: Leda" },
  { "name": "Orus", "description": "Gemini TTS voice: Orus" },
  { "name": "Aoede", "description": "Gemini TTS voice: Aoede" }
]
```

---

## Errors

| Code | Message | Cause |
|------|---------|-------|
| `-32602` | Invalid parameters | Empty prompt/text, invalid voice, or invalid style |
| `-32603` | Internal error | API call failure, handler initialization failure |

**Validation Errors:**
- `prompt: Prompt cannot be empty` - Image generation requires a non-empty prompt
- `text: Text cannot be empty` - TTS requires non-empty text
- `voice: Invalid voice '...'` - Voice must be one of the available voices
- `style: Invalid style '...'` - Style must be one of the available styles
