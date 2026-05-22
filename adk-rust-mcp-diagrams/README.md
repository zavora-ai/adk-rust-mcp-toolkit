# adk-rust-mcp-diagrams

MCP server for structured diagram generation. Part of the ADK Rust MCP toolkit.

## Overview

Generate precise, editable diagrams from natural language descriptions. Outputs SVG, Mermaid, or PlantUML — not raster images. Diagrams are vector-based, scalable, and can be embedded directly in documentation.

## Example Output

<img src="https://raw.githubusercontent.com/zavora-ai/adk-rust-mcp-toolkit/main/docs/assets/diagram_example.svg" width="600" alt="CI/CD flowchart"/>

> **Prompt:** "CI/CD pipeline: developer pushes code, triggers build, runs tests, if tests pass deploy to staging, then deploy to production"

## Features

- **Natural Language → Diagrams** — Describe it in English, get a rendered SVG
- **8 Diagram Types** — Flowchart, sequence, class, state, ER, Gantt, mindmap, pie
- **Editable Output** — SVG, Mermaid source, or PlantUML source
- **Themes** — default, dark, forest, neutral
- **Deterministic** — Same input always produces same output

## Installation

```bash
cargo install adk-rust-mcp-diagrams
npm install -g @mermaid-js/mermaid-cli  # optional, for SVG rendering
```

## Configuration

```bash
export GEMINI_API_KEY=your-api-key  # from https://aistudio.google.com/apikey
```

## Tools

### diagram_generate

Generate a diagram from natural language.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `description` | string | Yes | — | Natural language description |
| `type` | string | No | "auto" | flowchart, sequence, class, state, er, gantt, mindmap, pie |
| `format` | string | No | "svg" | svg, mermaid, plantuml, png |
| `theme` | string | No | "default" | default, dark, forest, neutral |
| `output_file` | string | No | — | Save path |

### diagram_from_code

Render a diagram from Mermaid or PlantUML source.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `code` | string | Yes | — | Mermaid or PlantUML source |
| `syntax` | string | No | "mermaid" | mermaid, plantuml |
| `format` | string | No | "svg" | svg, png |
| `theme` | string | No | "default" | Rendering theme |
| `output_file` | string | No | — | Save path |

### diagram_to_code

Convert natural language to diagram source code (no rendering).

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `description` | string | Yes | — | Natural language description |
| `type` | string | No | "auto" | Diagram type hint |
| `syntax` | string | No | "mermaid" | mermaid, plantuml |

## Supported Diagram Types

| Type | Use Case | Example |
|------|----------|---------|
| flowchart | Process flows, decision trees | CI/CD pipeline, user signup |
| sequence | API interactions | OAuth flow, microservices |
| class | Object models | Database schema, class hierarchy |
| state | State machines | Order status, connection states |
| er | Entity-relationship | Database design |
| gantt | Timelines | Sprint planning, roadmaps |
| mindmap | Topic maps | Feature planning, brainstorming |
| pie | Proportions | Budget allocation, survey results |

## License

Apache-2.0
