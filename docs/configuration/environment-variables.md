# Environment Variables for Rust MCP

The Rust MCP server can be fine-tuned using the following environment variables. These can be set in your OS or directly in your MCP client's configuration.

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_ANALYZER_PATH` | Absolute path to the `rust-analyzer` executable. | `~/.cargo/bin/rust-analyzer` |
| `RUST_MCP_FULL_ANALYSIS` | If `true`, enables full analysis including proc-macros and build scripts. Set to `false` for faster startup. | `true` |
| `LOG_LEVEL` | Level of logging for the MCP server (debug, info, warn, error). | `info` |

## Setting Variables

### Linux / macOS
```bash
export RUST_ANALYZER_PATH=/usr/local/bin/rust-analyzer
```

### Windows (PowerShell)
```powershell
$env:RUST_ANALYZER_PATH="C:\bin\rust-analyzer.exe"
```

### In MCP Config (JSON)
Most clients support an `env` object:
```json
"env": {
  "RUST_ANALYZER_PATH": "/path/to/analyzer"
}
```
