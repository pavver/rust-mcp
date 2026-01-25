# Configuring Rust MCP for ESP32 (Custom Toolchains)

When working with custom Rust toolchains, such as the `esp` toolchain for Espressif chips (ESP32), you might encounter the following error:

`error: 'rust-analyzer' is not installed for the custom toolchain 'esp'`

This happens because `rustup` tries to find a version of `rust-analyzer` that matches your current toolchain. Since the `esp` toolchain doesn't include its own `rust-analyzer`, the proxy fails.

## The Solution: Using a Wrapper Script

To fix this, you can create a wrapper script that points to a stable version of `rust-analyzer` while running your MCP server.

### 1. Create `run-rust-mcp.sh`

Create a script in your project root to override the analyzer path:

```bash
#!/bin/bash
# Set the absolute path to a stable rust-analyzer, bypassing the rustup proxy
export RUST_ANALYZER_PATH="/home/your-user/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rust-analyzer"

# Get the directory where the script is located
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)

# Execute the actual server
exec "$SCRIPT_DIR/target/release/rustmcp"
```

*Note: Make sure to replace the path in `RUST_ANALYZER_PATH` with the actual path on your system. You can find it by running `rustup which rust-analyzer --toolchain stable`.*

Don't forget to make it executable: `chmod +x run-rust-mcp.sh`.

### 2. Configure Gemini CLI (`gemini-extension.json`)

Use the script as the entry point for your MCP server in your `gemini-extension.json`:

```json
{
  "name": "rust-mcp-server",
  "version": "0.1.0",
  "contextFileName": "TOOLS_INSTRUCTION.md",
  "mcpServers": [
    {
      "command": "/home/your-user/rust/rust-mcp/run-rust-mcp.sh"
    }
  ]
}
```

### 3. Configure Other Clients

For Claude Desktop or Roo-Code, simply point the `command` field to the absolute path of your `run-rust-mcp.sh` script instead of the `rustmcp` binary.

## Why this works

By setting `RUST_ANALYZER_PATH` to an absolute path of a specific binary (e.g., from the `stable` toolchain), the MCP server will use that binary directly instead of calling `rust-analyzer` through the `rustup` proxy. This analyzer is perfectly capable of analyzing ESP32 projects as long as they have a valid `Cargo.toml`.
