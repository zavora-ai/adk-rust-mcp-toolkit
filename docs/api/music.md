# Music Server API

## Tools

### music_generate

Generate music from text prompts using Vertex AI Lyria.

#### Request Schema

```json
{
  "type": "object",
  "required": ["prompt"],
  "properties": {
    "prompt": {
      "type": "string",
      "description": "Text prompt describing the music to generate"
    },
    "negative_prompt": {
      "type": "string",
      "description": "What to avoid in the generated music"
    },
    "seed": {
      "type": "integer",
      "description": "Random seed for reproducible generation"
    },
    "sample_count": {
      "type": "integer",
      "description": "Number of samples to generate",
      "default": 1,
      "minimum": 1,
      "maximum": 4
    },
    "output_file": {
      "type": "string",
      "description": "Local file path to save WAV audio"
    },
    "output_gcs_uri": {
      "type": "string",
      "description": "GCS URI to upload WAV audio (gs://bucket/path)",
      "pattern": "^gs://[a-z0-9][a-z0-9._-]*/.*$"
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

**GCS Output** (when `output_gcs_uri` specified):

```json
{
  "content": [
    {
      "type": "text",
      "text": "Audio uploaded to: gs://bucket/path/output.wav"
    }
  ]
}
```

**Multiple Samples:**

When `sample_count > 1`, files are saved with index suffixes:
- `output_0.wav`, `output_1.wav`, etc.

#### Errors

| Code | Message | Description |
|------|---------|-------------|
| -32602 | Invalid params: prompt cannot be empty | Empty prompt provided |
| -32602 | Invalid params: sample_count must be between 1 and 4 | Invalid sample count |
| -32602 | Invalid params: output_gcs_uri must start with gs:// | Invalid GCS URI format |
| -32603 | API error | Vertex AI Lyria API failure |
| -32603 | No audio samples returned | API returned empty response |

## Resources

The music server does not expose any resources.

## Output Handling

### Priority

1. If `output_gcs_uri` is specified → Upload to GCS
2. Else if `output_file` is specified → Save to local file
3. Else → Return base64-encoded data

### Multiple Samples

When generating multiple samples:
- Local files: `stem_0.ext`, `stem_1.ext`, etc.
- GCS URIs: `gs://bucket/path/stem_0.ext`, etc.

## Prompt Tips

**Good prompts:**
- "Upbeat jazz piano with walking bass and brushed drums"
- "Ambient electronic soundscape with soft pads and gentle arpeggios"
- "Acoustic folk guitar with fingerpicking pattern"

**Using negative prompts:**
- Exclude vocals: `"negative_prompt": "vocals, singing, lyrics"`
- Exclude instruments: `"negative_prompt": "drums, percussion"`
