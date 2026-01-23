# API Reference

This section contains detailed API documentation for all MCP servers.

## Servers

- [Image Server API](./image.md)
- [Video Server API](./video.md)
- [Multimodal Server API](./multimodal.md)
- Music Server API (coming soon)
- Speech Server API (coming soon)
- AVTool Server API (coming soon)

## Common Patterns

### Tool Response Format

All tools return results in MCP's standard format:

```json
{
  "content": [
    {
      "type": "text",
      "text": "Result message or JSON data"
    }
  ]
}
```

For binary data (images, audio, video):

```json
{
  "content": [
    {
      "type": "image",
      "data": "base64-encoded-data",
      "mimeType": "image/png"
    }
  ]
}
```

### Error Format

Errors follow MCP's error format:

```json
{
  "code": -32602,
  "message": "Invalid params: prompt cannot be empty"
}
```

Common error codes:
- `-32602` - Invalid parameters
- `-32603` - Internal error
- `-32001` - Resource not found

### Resource URI Schemes

Each server uses a unique URI scheme:

| Server | Scheme | Example |
|--------|--------|---------|
| Image | `image://` | `image://models` |
| Video | `video://` | `video://models` |
| Music | `music://` | `music://models` |
| Speech | `speech://` | `speech://voices` |
| Multimodal | `multimodal://` | `multimodal://language_codes` |

## Authentication

All servers require Google Cloud authentication. See [Configuration](../configuration.md) for setup.
