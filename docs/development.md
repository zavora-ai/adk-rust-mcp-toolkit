# Development Guide

## Project Structure

```
adk-rust-mcp/
├── .env                      # Environment configuration
├── .kiro/
│   ├── hooks/               # Automation hooks
│   ├── specs/               # Feature specifications
│   └── steering/            # Development guidelines
├── docs/                    # Documentation
├── examples/                # ADK agent examples
├── adk-rust-mcp-common/     # Shared library
├── adk-rust-mcp-image/      # Image generation server
├── adk-rust-mcp-video/      # Video generation server
├── adk-rust-mcp-music/      # Music generation server
├── adk-rust-mcp-speech/     # Speech synthesis server
├── adk-rust-mcp-multimodal/ # Multimodal generation server
└── adk-rust-mcp-avtool/     # Audio/video processing server
```

## Prerequisites

- Rust 2024 edition (1.85+)
- Google Cloud SDK
- FFmpeg (for avtool server)

## Building

```bash
# Build all
cargo build

# Build specific package
cargo build --package adk-rust-mcp-image

# Release build
cargo build --release
```

## Testing

### Unit Tests

```bash
# All unit tests
cargo test --lib

# Specific package
cargo test --package adk-rust-mcp-image --lib
```

### Integration Tests

Integration tests require valid GCP credentials and call real APIs.

```bash
# Run integration tests
cargo test --package adk-rust-mcp-image --test integration_test

# Skip integration tests (CI)
SKIP_INTEGRATION_TESTS=1 cargo test
```

### Property-Based Tests

```bash
# Run with proptest
cargo test --lib -- property_tests

# More iterations
PROPTEST_CASES=1000 cargo test --lib
```

## Adding a New Server

1. Create the crate structure:

```
adk-rust-mcp-{name}/
├── Cargo.toml
├── tests/
│   └── integration_test.rs
└── src/
    ├── lib.rs       # Library exports
    ├── main.rs      # Entry point
    ├── handler.rs   # Business logic
    ├── resources.rs # MCP resources (if any)
    └── server.rs    # ServerHandler impl
```

2. Add to workspace `Cargo.toml`:

```toml
[workspace]
members = [
    # ...
    "adk-rust-mcp-{name}",
]
```

3. Implement the `ServerHandler` trait following patterns in `.kiro/steering/rmcp-server-patterns.md`

4. Add documentation:
   - `docs/servers/{name}.md`
   - `docs/api/{name}.md`

5. Update `docs/README.md` and `docs/api/README.md`

## rmcp 0.14 API

The workspace uses rmcp 0.14. Key types:

```rust
use rmcp::{
    model::{
        CallToolResult, Content, ListResourcesResult, ReadResourceResult,
        ResourceContents, ServerCapabilities, ServerInfo,
        PaginatedRequestParams, CallToolRequestParams, ReadResourceRequestParams,
    },
    ErrorData as McpError, ServerHandler,
};
```

See `.kiro/steering/rmcp-server-patterns.md` for complete implementation patterns.

## Code Style

- Use `rustfmt` for formatting
- Use `clippy` for linting
- Follow patterns in `.kiro/steering/rmcp-server-patterns.md`

```bash
# Format
cargo fmt

# Lint
cargo clippy --workspace --all-targets
```

## Running Servers Locally

### Stdio Mode (Default)

```bash
./target/release/adk-rust-mcp-image
```

### HTTP Mode

```bash
./target/release/adk-rust-mcp-image --transport http --port 8080
```

### Testing HTTP Endpoint

```bash
curl http://localhost:8080/mcp -X POST \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}'
```

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run --package adk-rust-mcp-image
```

### Test with MCP Inspector

```bash
# Install MCP inspector
npm install -g @anthropic/mcp-inspector

# Run server with inspector
mcp-inspector ./target/debug/adk-rust-mcp-image
```

## Examples Development

The examples use local path dependencies to `adk-rust`:

```toml
[dependencies]
adk-agent = { path = "../../../adk-rust/adk-agent" }
adk-tool = { path = "../../../adk-rust/adk-tool", features = ["http-transport"] }
```

To test examples:

```bash
# Terminal 1: Start server
./target/release/adk-rust-mcp-image --transport http --port 8080

# Terminal 2: Run example
cd examples/image-agent
cargo run --release
```

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run all tests
4. Build release binaries
5. Update documentation
6. Tag release

```bash
# Version bump
cargo set-version 0.2.0

# Full test
cargo test --workspace

# Release build
cargo build --release

# Tag
git tag v0.2.0
git push origin v0.2.0
```
