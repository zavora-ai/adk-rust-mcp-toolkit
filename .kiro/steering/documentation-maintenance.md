---
inclusion: manual
---

# Documentation Maintenance

This steering document provides guidance for maintaining documentation in the ADK Rust MCP workspace.

## Documentation Structure

The workspace documentation follows this structure:

```
docs/
├── README.md              # Main overview and quick start
├── configuration.md       # Environment variables and settings
├── development.md         # Contributing guide
├── api/
│   ├── README.md          # API reference overview
│   ├── image.md           # Image server API
│   ├── video.md           # Video server API
│   ├── music.md           # Music server API
│   ├── speech.md          # Speech server API
│   ├── multimodal.md      # Multimodal server API
│   └── avtool.md          # AVTool server API
└── servers/
    ├── image.md           # Image server guide
    ├── video.md           # Video server guide
    ├── music.md           # Music server guide
    ├── speech.md          # Speech server guide
    ├── multimodal.md      # Multimodal server guide
    └── avtool.md          # AVTool server guide
```

## When to Audit Documentation

Run the **Audit Documentation Completeness** hook when:

1. A new MCP server crate is added to the workspace
2. Significant changes are made to existing server tools or resources
3. Before a release to ensure all docs are up-to-date
4. When you notice documentation gaps

## How to Trigger the Audit Hook

The `audit-documentation` hook is a manually triggered hook. To run it:

1. Open the **Agent Hooks** section in the Kiro explorer view
2. Find **"Audit Documentation Completeness"**
3. Click the play/run button to trigger it

Alternatively, ask the agent: "Run the audit documentation hook"

## Documentation Standards

### Server Documentation (`docs/servers/*.md`)

Each server doc should include:
- Features overview
- Tools with parameters table
- Resources (if any)
- Configuration/environment variables
- Usage examples
- Error handling

### API Documentation (`docs/api/*.md`)

Each API doc should include:
- Request schema (JSON Schema format)
- Response format with examples
- Error codes and messages
- Validation constraints

## Keeping Documentation in Sync

The workspace has automated hooks that trigger on file changes:

| Hook | Trigger | Action |
|------|---------|--------|
| `update-api-docs` | `handler.rs`, `resources.rs` edited | Update API docs |
| `update-server-docs` | `server.rs`, `main.rs` edited | Update server docs |
| `update-readme-new-server` | New `Cargo.toml` created | Create initial docs |
| `audit-documentation` | Manual trigger | Full audit and gap fill |

## Checklist for New Servers

When adding a new MCP server:

- [ ] Create `docs/servers/{name}.md` with features, tools, resources, config
- [ ] Create `docs/api/{name}.md` with request/response schemas
- [ ] Add server to table in `docs/README.md`
- [ ] Add link in `docs/api/README.md`
- [ ] Verify with audit hook
