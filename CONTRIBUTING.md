# Contributing to ADK Rust MCP Toolkit

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Getting Started

1. Fork the repository
2. Clone your fork locally
3. Set up the development environment:

```bash
# Install Rust 2024 edition (1.85+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update

# Clone and build
git clone https://github.com/anthropics/adk-rust-mcp
cd adk-rust-mcp
cargo build
```

## Development Workflow

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation updates
- `refactor/description` - Code refactoring

### Making Changes

1. Create a new branch from `main`
2. Make your changes
3. Write or update tests
4. Update documentation if needed
5. Run tests and linting
6. Submit a pull request

### Code Style

- Follow Rust idioms and conventions
- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common issues
- Add documentation comments for public APIs

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --workspace --all-targets

# Run tests
cargo test --workspace
```

## Project Structure

Each MCP server follows this structure:

```
adk-rust-mcp-{name}/
├── Cargo.toml
├── src/
│   ├── lib.rs       # Library exports
│   ├── main.rs      # Entry point
│   ├── handler.rs   # Business logic
│   ├── resources.rs # MCP resources (if any)
│   └── server.rs    # ServerHandler impl
└── tests/
    └── integration_test.rs
```

## Adding a New Server

1. Create the crate directory: `adk-rust-mcp-{name}/`
2. Add to workspace in root `Cargo.toml`
3. Implement following the patterns in `.kiro/steering/rmcp-server-patterns.md`
4. Add documentation:
   - `docs/servers/{name}.md`
   - `docs/api/{name}.md`
5. Update `docs/README.md` and `docs/api/README.md`

## Testing

### Unit Tests

Add unit tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }
}
```

### Property-Based Tests

Use proptest for validation logic:

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn property_name(value in strategy()) {
            prop_assert!(condition);
        }
    }
}
```

### Integration Tests

Integration tests go in `tests/integration_test.rs` and test against real APIs:

```bash
# Run integration tests (requires valid credentials)
cargo test --package adk-rust-mcp-image --test integration_test

# Skip integration tests
SKIP_INTEGRATION_TESTS=1 cargo test
```

## Documentation

- Update relevant docs when changing functionality
- Follow the documentation standards in `docs/documentation-maintenance.md`
- Server docs go in `docs/servers/`
- API docs go in `docs/api/`

## Pull Request Process

1. Ensure all tests pass
2. Update documentation
3. Update CHANGELOG.md
4. Add a clear PR description
5. Link any related issues
6. Request review from maintainers

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the code, not the person
- Help others learn and grow

## Questions?

Open an issue for questions or discussions about contributing.
