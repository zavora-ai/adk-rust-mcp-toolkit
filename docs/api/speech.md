# Speech Server API

## Tools

### speech_synthesize

Convert text to speech using Google Cloud TTS Chirp3-HD voices.

#### Request Schema

```json
{
  "type": "object",
  "required": ["text"],
  "properties": {
    "text": {
      "type": "string",
      "description": "Text to synthesize into speech"
    },
    "voice": {
      "type": "string",
      "description": "Voice name (Chirp3-HD voice)",
      "default": "en-US-Chirp3-HD-Achernar"
    },
    "language_code": {
      "type": "string",
      "description": "Language code (e.g., 'en-US')",
      "default": "en-US"
    },
    "speaking_rate": {
      "type": "number",
      "description": "Speaking rate multiplier",
      "default": 1.0,
      "minimum": 0.25,
      "maximum": 4.0
    },
    "pitch": {
      "type": "number",
      "description": "Pitch adjustment in semitones",
      "default": 0.0,
      "minimum": -20.0,
      "maximum": 20.0
    },
    "pronunciations": {
      "type": "array",
      "description": "Custom pronunciations for specific words",
      "items": {
        "type": "object",
        "required": ["word", "phonetic", "alphabet"],
        "properties": {
          "word": {
            "type": "string",
            "description": "Word to apply pronunciation to"
          },
          "phonetic": {
            "type": "string",
            "description": "Phonetic representation"
          },
          "alphabet": {
            "type": "string",
            "description": "Phonetic alphabet",
            "enum": ["ipa", "x-sampa"]
          }
        }
      }
    },
    "output_file": {
      "type": "string",
      "description": "Local file path to save WAV audio"
    }
  }
}
```

#### Response

**Base64 Output** (default):

```json
{
  "content": [
    {
      "type": "text",
      "text": "data:audio/wav;base64,UklGRi..."
    }
  ]
}
```

**Local File Output** (when `output_file` specified):

```json
{
  "content": [
    {
      "type": "text",
      "text": "Audio saved to: /path/to/output.wav"
    }
  ]
}
```

#### Errors

| Code | Message | Description |
|------|---------|-------------|
| -32602 | Invalid params: text cannot be empty | Empty text provided |
| -32602 | Invalid params: speaking_rate must be between 0.25 and 4.0 | Rate out of range |
| -32602 | Invalid params: pitch must be between -20.0 and 20.0 | Pitch out of range |
| -32602 | Invalid params: invalid alphabet | Pronunciation alphabet not ipa or x-sampa |
| -32603 | API error | Cloud TTS API failure |

---

### speech_list_voices

List available Chirp3-HD voices.

#### Request Schema

```json
{
  "type": "object",
  "properties": {}
}
```

#### Response

```json
{
  "content": [
    {
      "type": "text",
      "text": "[{\"name\": \"en-US-Chirp3-HD-Achernar\", \"language_codes\": [\"en-US\"], ...}]"
    }
  ]
}
```

## Resources

The speech server does not expose any resources.

## SSML Support

When pronunciations are provided, the text is automatically wrapped in SSML with phoneme elements:

```xml
<speak>I like <phoneme alphabet="ipa" ph="təˈmeɪtoʊ">tomato</phoneme> soup</speak>
```

## Phonetic Alphabets

### IPA (International Phonetic Alphabet)

Standard phonetic notation used in dictionaries.

Example: `təˈmeɪtoʊ` for "tomato" (American pronunciation)

### X-SAMPA

ASCII-compatible phonetic alphabet.

Example: `t@"meItoU` for "tomato" (American pronunciation)
