# Configuring Rust MCP for Gemini CLI

The Gemini CLI can use MCP servers as extensions to provide specialized tools for your tasks.

## Configuration

To use Rust MCP with Gemini CLI, you need to add it to your `gemini-extension.json` or link it directly.

### Option 1: Linking as an Extension

If you are developing or using this repository as a Gemini extension:

```bash
cd rust-mcp
gemini extensions link .
```

### Option 2: Manual Tool Configuration

In your Gemini CLI configuration (usually `~/.gemini/config.json` or project-specific `.gemini/gemini-extension.json`):

```json
{
  "mcpServers": {
    "rust-mcp": {
      "command": "/path/to/rust-mcp/target/release/rustmcp",
      "args": []
    }
  }
}
```

## Using with Gemini CLI

Once configured, you can use the tools via the interactive agent.

**Example command:**
> "Analyze the current project and show me any compiler errors."

The agent will automatically use `run_cargo_check` or `get_diagnostics` from the Rust MCP server.
