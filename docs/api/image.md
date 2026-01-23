# Image Server API

## Tools

### image_generate

Generate images from text prompts using Vertex AI Imagen.

#### Request Schema

```json
{
  "type": "object",
  "required": ["prompt"],
  "properties": {
    "prompt": {
      "type": "string",
      "description": "Text prompt describing the image to generate",
      "maxLength": 2000
    },
    "negative_prompt": {
      "type": "string",
      "description": "What to avoid in the generated image"
    },
    "model": {
      "type": "string",
      "description": "Model to use for generation",
      "default": "imagen-4.0-generate-preview-06-06",
      "enum": [
        "imagen-4.0-generate-preview-06-06",
        "imagen-3.0-generate-002",
        "imagen-3.0-fast-generate-001",
        "imagen-4",
        "imagen-3",
        "imagen-3-fast"
      ]
    },
    "aspect_ratio": {
      "type": "string",
      "description": "Aspect ratio for the generated image",
      "default": "1:1",
      "enum": ["1:1", "3:4", "4:3", "9:16", "16:9"]
    },
    "number_of_images": {
      "type": "integer",
      "description": "Number of images to generate",
      "default": 1,
      "minimum": 1,
      "maximum": 4
    },
    "seed": {
      "type": "integer",
      "description": "Random seed for reproducible generation (not supported with watermark)"
    },
    "output_file": {
      "type": "string",
      "description": "Local file path to save the image"
    },
    "output_uri": {
      "type": "string",
      "description": "GCS URI to upload the image (gs://bucket/path)",
      "pattern": "^gs://[a-z0-9][a-z0-9._-]*[a-z0-9]/.*$"
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
      "type": "image",
      "data": "iVBORw0KGgoAAAANSUhEUgAA...",
      "mimeType": "image/png"
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
      "text": "Images saved to: /path/to/image.png"
    }
  ]
}
```

**GCS Output** (when `output_uri` specified):

```json
{
  "content": [
    {
      "type": "text",
      "text": "Images uploaded to: gs://bucket/path/image.png"
    }
  ]
}
```

#### Errors

| Code | Message | Description |
|------|---------|-------------|
| -32602 | Invalid params: prompt cannot be empty | Empty prompt provided |
| -32602 | Invalid params: prompt length exceeds maximum | Prompt too long for model |
| -32602 | Invalid params: invalid aspect ratio | Unsupported aspect ratio |
| -32602 | Invalid params: number_of_images must be 1-4 | Invalid image count |
| -32603 | API error | Vertex AI API failure |

## Resources

### image://models

List available image generation models.

#### Response

```json
[
  {
    "id": "imagen-4.0-generate-preview-06-06",
    "aliases": ["imagen-4", "imagen-4.0", "imagen4", "imagen-4-preview"],
    "max_prompt_length": 2000,
    "supported_aspect_ratios": ["1:1", "3:4", "4:3", "9:16", "16:9"],
    "max_images": 4
  },
  {
    "id": "imagen-3.0-generate-002",
    "aliases": ["imagen-3", "imagen-3.0", "imagen3"],
    "max_prompt_length": 480,
    "supported_aspect_ratios": ["1:1", "3:4", "4:3", "9:16", "16:9"],
    "max_images": 4
  },
  {
    "id": "imagen-3.0-fast-generate-001",
    "aliases": ["imagen-3-fast", "imagen-3.0-fast"],
    "max_prompt_length": 480,
    "supported_aspect_ratios": ["1:1", "3:4", "4:3", "9:16", "16:9"],
    "max_images": 4
  }
]
```

### image://providers

List available image generation providers.

#### Response

```json
[
  {
    "id": "google-imagen",
    "name": "Google Imagen",
    "description": "Google's Vertex AI Imagen API for high-quality image generation",
    "is_default": true
  }
]
```

### image://segmentation_classes

List segmentation classes for image editing operations.

#### Response

```json
[
  {"id": "background", "name": "Background", "description": "The background of the image"},
  {"id": "person", "name": "Person", "description": "Human figures in the image"},
  {"id": "face", "name": "Face", "description": "Human faces in the image"},
  {"id": "hair", "name": "Hair", "description": "Hair on human figures"},
  {"id": "clothing", "name": "Clothing", "description": "Clothing and accessories"},
  {"id": "sky", "name": "Sky", "description": "Sky regions in the image"},
  {"id": "ground", "name": "Ground", "description": "Ground or floor surfaces"},
  {"id": "vegetation", "name": "Vegetation", "description": "Plants, trees, and vegetation"},
  {"id": "building", "name": "Building", "description": "Buildings and structures"},
  {"id": "vehicle", "name": "Vehicle", "description": "Cars, trucks, and vehicles"},
  {"id": "animal", "name": "Animal", "description": "Animals in the image"},
  {"id": "food", "name": "Food", "description": "Food items in the image"},
  {"id": "furniture", "name": "Furniture", "description": "Furniture and home items"}
]
```
