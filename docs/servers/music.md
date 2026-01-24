# Music Server (adk-rust-mcp-music)

MCP server for music generation using Google Vertex AI Lyria API.

## Features

- Text-to-music generation with Lyria model
- Batch generation (1-4 samples per request)
- Negative prompts for refined control
- Output to base64, local files, or GCS
- Reproducible generation with seed parameter

## Tools

### music_generate

Generate music from text prompts.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | Yes | - | Text description of the music to generate |
| `negative_prompt` | string | No | - | What to avoid in the generated music |
| `seed` | integer | No | - | Random seed for reproducibility |
| `sample_count` | integer | No | `1` | Number of samples to generate (1-4) |
| `output_file` | string | No | - | Local file path to save WAV |
| `output_gcs_uri` | string | No | - | GCS URI to upload WAV |

**Example:**

```json
{
  "prompt": "Upbeat jazz piano with walking bass and brushed drums",
  "negative_prompt": "vocals, lyrics, singing",
  "sample_count": 2,
  "output_file": "/tmp/jazz.wav"
}
```

**Response:**

Returns base64-encoded WAV data, local file paths, or GCS URIs depending on output parameters.

## Resources

The music server does not expose any resources.

## Models

| Model ID | Description |
|----------|-------------|
| `lyria-realtime-singing` | Google's Lyria model for music generation |

## Usage Examples

### Basic Generation

```bash
mcp call music_generate '{"prompt": "A calm ambient soundscape"}'
```

### Save to Local File

```bash
mcp call music_generate '{
  "prompt": "Electronic dance music with heavy bass",
  "output_file": "/tmp/edm.wav"
}'
```

### Upload to GCS

```bash
mcp call music_generate '{
  "prompt": "Classical orchestral piece",
  "output_gcs_uri": "gs://my-bucket/music/classical.wav"
}'
```

### Multiple Samples

```bash
mcp call music_generate '{
  "prompt": "Lo-fi hip hop beat",
  "sample_count": 4,
  "seed": 42
}'
```

## Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PROJECT_ID` | Yes | - | GCP project ID |
| `LOCATION` | No | `us-central1` | GCP region |
| `GCS_BUCKET` | No | - | Default GCS bucket |

## Error Handling

| Error | Description |
|-------|-------------|
| `INVALID_PARAMS` | Invalid parameters (empty prompt, invalid sample_count) |
| `API_ERROR` | Vertex AI Lyria API error |
| `AUTH_ERROR` | Authentication failed |
| `GCS_ERROR` | GCS upload failed |
