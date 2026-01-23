# Development Guide

## Project Structure

```
adk-rust-mcp/
├── .env                      # Environment configuration
├── .kiro/
│   ├── specs/               # Feature specifications
│   └── steering/            # Development guidelines
├── docs/                    # Documentation
├── adk-rust-mcp-common/     # Shared library
├── adk-rust-mcp-image/      # Image generation server
├── adk-rust-mcp-video/      # Video generation server
├── adk-rust-mcp-music/      # Music generation server
├── adk-rust-mcp-speech/     # Speech synthesis server
├── adk-rust-mcp-multimodal/ # Multimodal generation server
└── adk-rust-mcp-avtool/     # Audio/video processing server
```

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
```

## Adding a New Server

1. Create the crate structure:

```
adk-rust-mcp-{name}/
├── Cargo.toml
├── tests/
│   └── integration_test.rs
└── src/
    ├── lib.rs
    ├── main.rs
    ├── handler.rs
    ├── resources.rs
    └── server.rs
```

2. Add to workspace `Cargo.toml`:

```toml
[workspace]
members = [
    # ...
    "adk-rust-mcp-{name}",
]
```

3. Implement the `ServerHandler` trait (see steering docs).

4. Add integration tests.

5. Update documentation.

## Code Style

- Use `rustfmt` for formatting
- Use `clippy` for linting
- Follow patterns in `.kiro/steering/rmcp-server-patterns.md`

```bash
# Format
cargo fmt

# Lint
cargo clippy
```

## Documentation

Documentation is auto-generated when source files change. See hooks in `.kiro/hooks/`.

Manual generation:

```bash
# Generate API docs
cargo doc --no-deps --open
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

## Release Process

1. Update version in `Cargo.toml`
2. Run all tests
3. Build release binaries
4. Update documentation
5. Tag release

```bash
# Version bump
cargo set-version 0.2.0

# Full test
cargo test

# Release build
cargo build --release
```
