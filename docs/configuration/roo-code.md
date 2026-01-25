# Configuring Rust MCP for Roo-Code (Roo-Cline)

Roo-Code is a powerful VS Code extension that supports MCP servers for enhanced coding capabilities.

## Configuration Steps

1. Open VS Code.
2. Go to the Roo-Code settings or open the configuration file manually.
3. The configuration is typically stored in `~/.roo/config.json` (or via the UI).

## Configuration Example

```json
{
  "mcp_servers": [
    {
      "name": "rust-analyzer",
      "command": "/path/to/rust-mcp/target/release/rustmcp",
      "args": [],
      "env": {
        "RUST_ANALYZER_PATH": "/custom/path/to/rust-analyzer",
        "RUST_MCP_FULL_ANALYSIS": "true"
      }
    }
  ]
}
```

## Tips for Roo-Code

- **Semantic Navigation**: Roo-Code excels at using `find_definition` and `find_references` to understand complex Rust codebases.
- **Diagnostics**: Use `get_diagnostics` or `run_cargo_check` to let Roo-Code see compiler errors directly in the chat interface.
- **Context Management**: Since Rust projects can be large, encourage Roo-Code to use `document_symbols` to explore file structures efficiently.
