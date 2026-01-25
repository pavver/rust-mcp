# Configuring Rust MCP for Claude Desktop

Claude Desktop allows you to integrate MCP servers to give Claude access to local tools and intelligence.

## Configuration File Location

Depending on your operating system, open the `claude_desktop_config.json` file:

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

## Configuration Example

Add the `rust-analyzer` server to the `mcpServers` section. Replace `/path/to/rust-mcp/` with the absolute path to your cloned repository.

```json
{
  "mcpServers": {
    "rust-analyzer": {
      "command": "/path/to/rust-mcp/target/release/rustmcp",
      "args": [],
      "env": {
        "RUST_ANALYZER_PATH": "/custom/path/to/rust-analyzer",
        "RUST_MCP_FULL_ANALYSIS": "true"
      }
    }
  }
}
```

### Parameters

- `command`: The absolute path to the compiled `rustmcp` binary.
- `args`: Leave empty as the server uses stdio by default.
- `env`:
  - `RUST_ANALYZER_PATH`: (Optional) Path to your `rust-analyzer` binary. Default is `~/.cargo/bin/rust-analyzer`.
  - `RUST_MCP_FULL_ANALYSIS`: (Optional) Set to `false` to speed up initialization on very large projects.

## Troubleshooting

1. **Path Issues**: Always use absolute paths. Claude Desktop cannot resolve `~/` or relative paths.
2. **Binary Permissions**: Ensure the binary is executable: `chmod +x target/release/rustmcp`.
3. **Logs**: You can check Claude Desktop logs if the server fails to start.
