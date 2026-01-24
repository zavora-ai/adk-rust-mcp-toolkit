# Speech Server (adk-rust-mcp-speech)

MCP server for text-to-speech synthesis using Google Cloud TTS Chirp3-HD API.

## Features

- High-quality text-to-speech with Chirp3-HD voices
- Adjustable speaking rate and pitch
- Custom pronunciations using IPA or X-SAMPA phonetic alphabets
- SSML support for advanced speech control
- Output to base64 or local WAV files

## Tools

### speech_synthesize

Convert text to speech.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `text` | string | Yes | - | Text to synthesize |
| `voice` | string | No | `en-US-Chirp3-HD-Achernar` | Voice name |
| `language_code` | string | No | `en-US` | Language code |
| `speaking_rate` | float | No | `1.0` | Speaking rate (0.25-4.0) |
| `pitch` | float | No | `0.0` | Pitch in semitones (-20.0 to 20.0) |
| `pronunciations` | array | No | - | Custom pronunciations |
| `output_file` | string | No | - | Local file path to save WAV |

**Pronunciation Object:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `word` | string | Yes | Word to apply pronunciation to |
| `phonetic` | string | Yes | Phonetic representation |
| `alphabet` | string | Yes | Alphabet: "ipa" or "x-sampa" |

**Example:**

```json
{
  "text": "Hello, welcome to our application!",
  "voice": "en-US-Chirp3-HD-Achernar",
  "speaking_rate": 1.2,
  "pitch": 2.0,
  "output_file": "/tmp/greeting.wav"
}
```

**With Custom Pronunciation:**

```json
{
  "text": "I love tomato soup",
  "pronunciations": [
    {
      "word": "tomato",
      "phonetic": "təˈmeɪtoʊ",
      "alphabet": "ipa"
    }
  ]
}
```

### speech_list_voices

List available Chirp3-HD voices.

**Parameters:** None

**Response:**

```json
[
  {
    "name": "en-US-Chirp3-HD-Achernar",
    "language_codes": ["en-US"],
    "ssml_gender": "FEMALE",
    "natural_sample_rate_hertz": 24000
  }
]
```

## Resources

The speech server does not expose any resources.

## Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PROJECT_ID` | Yes | - | GCP project ID |

## Usage Examples

### Basic Synthesis

```bash
mcp call speech_synthesize '{"text": "Hello world"}'
```

### Save to File

```bash
mcp call speech_synthesize '{
  "text": "This is a test of text to speech.",
  "output_file": "/tmp/speech.wav"
}'
```

### Adjust Rate and Pitch

```bash
mcp call speech_synthesize '{
  "text": "Speaking faster and higher",
  "speaking_rate": 1.5,
  "pitch": 5.0
}'
```

### List Voices

```bash
mcp call speech_list_voices '{}'
```

## Error Handling

| Error | Description |
|-------|-------------|
| `INVALID_PARAMS` | Invalid parameters (empty text, rate/pitch out of range) |
| `API_ERROR` | Cloud TTS API error |
| `AUTH_ERROR` | Authentication failed |

## Validation Constraints

| Parameter | Constraint |
|-----------|------------|
| `text` | Cannot be empty |
| `speaking_rate` | 0.25 to 4.0 |
| `pitch` | -20.0 to 20.0 semitones |
| `alphabet` | Must be "ipa" or "x-sampa" |
